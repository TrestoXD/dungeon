package com.vit.dungeon.graphics;

import com.vit.dungeon.Game;
import com.vit.dungeon.input.Controller;

public class Render3D extends Render {

	public double[] zBuffer;
	private double renderdistance = 5000;
	private int spriteSheetWidth = 16;
	private double forward, right, up, cosine, sine, walking, yLook;
	

	public Render3D(int width, int height) {
		super(width, height);
		zBuffer = new double[width * height];
	}

	public void floor(Game game) {

		double floorPosition = 8;
		double ceilingPosition = 8;
		forward = game.controls.z;
		right = game.controls.x;
		up = game.controls.y;
		walking = 0;

	    //yLook = game.controls.ylook;
		double rotation = game.controls.rotation;
		cosine = Math.cos(rotation);
		sine = Math.sin(rotation);

		for (int y = 0; y < height; y++) {
			double ceiling = ((y - height / 2.0) - yLook) / height;
			double z = (floorPosition + up) / ceiling;
			
			if (Controller.walk) {
				walking = Math.sin(game.time / 6.0) * 0.5;
				z = (floorPosition + up + walking) / ceiling;
			}
			if (Controller.crouchWalk && Controller.walk) {
				walking = Math.sin(game.time / 6.0) * 0.2;
				z = (floorPosition + up + walking) / ceiling;
			}
			if (Controller.sprintWalk && Controller.walk) {
				walking = Math.sin(game.time / 12.0) * 1;
				z = (floorPosition + up + walking) / ceiling;
			}
			
			if (ceiling == 0)
				ceiling = 0.0001;

			if (ceiling < 0) {
				z = (ceilingPosition - up) / -ceiling;
				if (Controller.walk) {
					z = (ceilingPosition - up - walking) / -ceiling;
				}
			}

			for (int x = 0; x < width; x++) {
				double depth = (x - width / 2.0) / height;
				depth *= z;
				double xx = depth * cosine + z * sine;
				double yy = z * cosine - depth * sine;
				int xPix = (int) ((xx + right) * 2);
				int yPix = (int) ((yy + forward) * 2);
				zBuffer[x + y * width] = z;
				pixels[x + y * width] = Texture.block.pixels[(xPix & 15) + (yPix & 15) * spriteSheetWidth];

				if (z > 500) {
					pixels[x + y * width] = 0;
				}
			}
		}
	}

	public void renderWall (double xLeft, double xRight, double zDistanceLeft, double zDistanceRight, double yHeight) {
		double xcLeft = ((xLeft) - (right / 16)) * 2;
		double zcLeft = ((zDistanceLeft) - (forward / 16)) * 2;

		double rotLeftSideX = xcLeft * cosine - zcLeft * sine;
		double yCornerTL = ( (-yHeight) - (-up / 16 + -(walking / 16))) * 2;
		double yCornerBL = ((+0.5 - yHeight) - (-up / 16 + -(walking / 16))) * 2;
		double rotLeftSideZ = zcLeft * cosine + xcLeft * sine;

		double xcRight = ((xRight) - (right / 16)) * 2;
		double zcRight = ((zDistanceRight) - (forward / 16)) * 2;

		double rotRightSideX = xcRight * cosine - zcRight * sine;
		double yCornerTR = ((-yHeight) - (-up / 16 + -(walking / 16))) * 2;
		double yCornerBR = ((+0.5 - yHeight) - (-up / 16 + -(walking / 16))) * 2;
		double rotRightSideZ = zcRight * cosine + xcRight * sine;

		double xPixelLeft = (rotLeftSideX / rotLeftSideZ * height + width / 2);
		double xPixelRight = (rotRightSideX / rotRightSideZ * height + width / 2);

		if (xPixelLeft >= xPixelRight) {
		return;
		}

		int xPixelLeftInt = (int) (xPixelLeft);
		int xPixelRightInt = (int) (xPixelRight);

		if (xPixelLeftInt < 0) {
			xPixelLeftInt = 0;
		}
		if(xPixelRightInt > width) {
			xPixelRightInt = width;
		}
		
		double yPixelLeftTop = yCornerTL / rotLeftSideZ * height + height / 2.0;
		double yPixelLeftBottom = yCornerBL / rotLeftSideZ * height + height / 2.0;
		double yPixelRightTop = yCornerTR / rotRightSideZ * height + height / 2.0;
		double yPixelRightBottom = yCornerBR / rotRightSideZ * height + height / 2.0;
	
		double tex1 = 1 / rotLeftSideZ;
		double tex2 = 1 / rotRightSideZ;
		double tex3 = 0 / rotLeftSideZ;
		double tex4 = 16 / rotRightSideZ - tex3;
		
		for (int x = xPixelLeftInt; x < xPixelRightInt; x++) {
			double pixelRotation = (x - xPixelLeft) / (xPixelRight - xPixelLeft);
			
			int xTexture = (int) ((tex3 + tex4 * pixelRotation) / (tex1 + (tex2 - tex1) * pixelRotation));
			
			double yPixelTop = yPixelLeftTop + (yPixelRightTop - yPixelLeftTop) * pixelRotation;
			double yPixelBottom = yPixelLeftBottom + (yPixelRightBottom - yPixelLeftBottom) * pixelRotation;
		
			int yPixelTopInt = (int) (yPixelTop);
			int yPixelBottomInt = (int) (yPixelBottom);
			
			if(yPixelTopInt < 0) {
				yPixelTopInt = 0;
			}
			if(yPixelBottomInt > height) {
				yPixelBottomInt = height;
			}
			
			for (int y = yPixelTopInt; y < yPixelBottomInt; y++) {
				double pixelRotationY = (y-yPixelTop) / (yPixelBottom - yPixelTop);
				int yTexture = (int) (16 * pixelRotationY);
				pixels[x +y * width] = Texture.stone.pixels[(xTexture & 15) + (yTexture & 15) * 16]; 
				//pixels[x + y * width] = xTexture * 100 + yTexture * 100 * 256;
				zBuffer[x + y * width] = 1 / (tex1 + (tex2 - tex1) * pixelRotation) * 8;
			}
			
		}
	}
	
	public void renderDistanceLimiter() {
		for (int i = 0; i < width * height; i++) {
			int colour = pixels[i];
			int brightness = (int) (renderdistance / (zBuffer[i]));

			if (brightness < 0) {
				brightness = 0;
			}

			if (brightness > 255) {
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
