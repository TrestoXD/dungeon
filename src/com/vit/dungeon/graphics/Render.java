package com.vit.dungeon.graphics;

public class Render {
	public final int width;
	public final int height;
	public final int[] pixels;
	
	public Render(int width, int height) {
		this.width = width;
		this.height = height;
		pixels = new int[width * height];
	}
	
	public void draw(Render render, int xOffs, int yOffs) {
		
		int startY = Math.max(0, -yOffs);
		int endY = Math.min(render.height, height - yOffs);
		
		int startX = Math.max(0, -xOffs);
		int endX = Math.min(render.width, width - xOffs);
		
		
		if (startY >= endY || startX >= endX) {
			return;
		}
		
		
		for (int y = startY; y < endY; y++) {
			int yPix = y + yOffs;
			int targetRowOffset = yPix * width;
			int sourceRowOffset = y * render.width;
			
			for (int x = startX; x < endX; x++) {
				int alpha = render.pixels[x + sourceRowOffset];
				
				
				if (alpha > 0) {
					pixels[(x + xOffs) + targetRowOffset] = alpha;
				}
			}
		}
	}
}
