package com.vit.dungeon.graphics;

import com.vit.dungeon.Game;

public class Render3D extends Render{

	public double[] zBuffer;
	private double renderdistance = 5000;
	private int spriteSheetWidth = 16;
	
	public Render3D(int width, int height) {
		super(width,height);
		zBuffer = new double[width*height];
	}
	
	public void floor(Game game) {
		
		double floorPosition = 8;
		double ceilingPosition = 10;
		double forward = game.controls.z;
		double right = game.controls.x;
		
		double ylook = game.controls.ylook;
		double rotation = game.controls.rotation;
		double cosine = Math.cos(rotation);
		double sine = Math.sin(rotation);
		
		for (int y = 0; y < height; y++) {
			double ceiling = (y - height / 2.0 - ylook) / height;
			
			if (ceiling == 0) ceiling = 0.0001; 
			
			double z = floorPosition / ceiling;
			
			if(ceiling  < 0) {
				z = ceilingPosition / -ceiling;
			}
			
			for (int x = 0; x < width; x++) {
				double depth = (x - width / 2.0) / height;
				depth *= z;
				double xx = depth * cosine + z * sine;
				double yy = z * cosine - depth * sine;
				int xPix = (int) ((xx + right) * 4);
				int yPix = (int) ((yy + forward) * 4);
				zBuffer[x + y * width] = z;
				pixels[x + y * width] = Texture.block.pixels[(xPix & 15) + (yPix & 15) * spriteSheetWidth];
				
				if(z > 500) {
					pixels[x + y * width] = 0;
				}
			}
		}
	}
	
	public void renderDistanceLimiter() {
		for( int i = 0; i < width * height; i++) {
			int colour = pixels[i];
			int brightness = (int) (renderdistance / (zBuffer[i]));
			
			if (brightness < 0) {
				brightness = 0;
			}
			
			if(brightness > 255) {
				brightness = 255;
			}
			
			int r = (colour >> 16) & 0xff;
			int g = (colour >> 8) & 0xff;
			int b = (colour) & 0xff;
		
			r = r * brightness / 255;
			g = g * brightness / 255;
			b = b * brightness / 255;
		
			pixels[i] = r << 16 | g << 8 | b;
		}
	}

	public void drawMap() {
		
	}
}
