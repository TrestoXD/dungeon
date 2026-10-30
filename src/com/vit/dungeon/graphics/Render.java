package com.vit.dungeon.graphics;

public class Render {
	public final int width;
	public final int height;
	public final int[] pixels;

	public Render(int width, int height) {
		this(width, height, new int[width * height]);
	}

	public Render(int width, int height, int[] pixels) {
		if (pixels.length != width * height) {
			throw new IllegalArgumentException("El buffer no coincide con " + width + "x" + height);
		}
		this.width = width;
		this.height = height;
		this.pixels = pixels;
	}

	public void draw(Render render, int xOffs, int yOffs) {
		int startY = Math.max(0, -yOffs);
		int endY = Math.min(render.height, height - yOffs);
		int startX = Math.max(0, -xOffs);
		int endX = Math.min(render.width, width - xOffs);

		if (startY >= endY || startX >= endX) return;

		for (int y = startY; y < endY; y++) {
			int dst = (y + yOffs) * width + startX + xOffs;
			int src = y * render.width + startX;
			for (int x = startX; x < endX; x++, dst++, src++) {
				int p = render.pixels[src];
				if ((p & 0xFFFFFF) != 0) pixels[dst] = p;
			}
		}
	}
}