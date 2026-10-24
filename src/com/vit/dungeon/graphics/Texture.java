package com.vit.dungeon.graphics;

import java.awt.image.BufferedImage;

import javax.imageio.ImageIO;

public class Texture {
	public static Render block = loadBitmap("/textures/cobblestone.png");
	public static Render stone = loadBitmap("/textures/stone.png");
	
	public static Render loadBitmap(String fileName) {
		try {
			BufferedImage image = ImageIO.read(Texture.class.getResource(fileName));
			int width = image.getWidth();
			int height = image.getHeight();
			Render result = new Render(width, height);
			image.getRGB(0, 0, width, height, result.pixels, 0, width);
			return result;
		} catch(Exception e) {
			System.out.println("TEXTURE NOT FOUND");
			throw new RuntimeException(e);
		}
	}
	
}
