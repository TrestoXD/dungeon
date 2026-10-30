package com.vit.dungeon.graphics;

import java.awt.image.BufferedImage;

import javax.imageio.ImageIO;

public class Texture {
	public static final int FOG_LEVELS = 32;

	public static final Render block = loadBitmap("/textures/cobblestone.png");
	public static final Render stone = loadBitmap("/textures/stone.png");

	public static final int[] blockFlat = buildShadeTable(block);
	public static final int[] stoneFlat = buildShadeTable(stone);

	public static Render loadBitmap(String fileName) {
		try {
			BufferedImage image = ImageIO.read(Texture.class.getResource(fileName));
			int width = image.getWidth();
			int height = image.getHeight();
			if (Integer.bitCount(width) != 1 || Integer.bitCount(height) != 1) {
				throw new IllegalArgumentException(fileName + " debe tener ancho y alto potencia de 2");
			}
			Render result = new Render(width, height);
			image.getRGB(0, 0, width, height, result.pixels, 0, width);
			return result;
		} catch (Exception e) {
			System.out.println("TEXTURE NOT FOUND: " + fileName);
			throw new RuntimeException(e);
		}
	}

	private static final float G_BASE = 0.50f, G_GAIN = 0.35f;
	private static final float B_BASE = 0.10f, B_GAIN = 0.25f;

	private static int[] buildShadeTable(Render tex) {
		int n = tex.pixels.length;
		int[] table = new int[FOG_LEVELS * n];
		for (int level = 0; level < FOG_LEVELS; level++) {
			float k = level / (float) (FOG_LEVELS - 1); // 0..1
			float fr = k;
			float fg = k * (G_BASE + G_GAIN * k);
			float fb = k * (B_BASE + B_GAIN * k);
			for (int i = 0; i < n; i++) {
				int c = tex.pixels[i];
				int r = (int) (((c >> 16) & 0xff) * fr);
				int g = (int) (((c >> 8) & 0xff) * fg);
				int b = (int) ((c & 0xff) * fb);
				table[level * n + i] = ((r & 0xF8) << 16) | ((g & 0xF8) << 8) | (b & 0xF8);
			}
		}
		return table;
	}

	static int tileShift(Render tex) {
		return log2(tex.pixels.length);
	}

	static int log2(int powerOfTwo) {
		return Integer.numberOfTrailingZeros(powerOfTwo);
	}
}