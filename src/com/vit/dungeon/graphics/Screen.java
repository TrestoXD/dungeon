package com.vit.dungeon.graphics;

import java.util.Random;

import com.vit.dungeon.Game;
import com.vit.dungeon.world.World;

public class Screen extends Render{
	private Render test;
	private Render3D render;
	
	public Screen(int width, int height) {
		super(width, height);
		Random random = new Random();
		render = new Render3D(width,height);
		test = new Render(256,256);
		for(int i = 0; i < 256*256; i++) {
			test.pixels[i] = random.nextInt() * (random.nextInt(5) / 4);
		}
	}
	
	public void render(Game game) {
		for (int i = 0; i <width * height; i++) {
			pixels[i] = 0; 
		}
		render.floor(game);
		render.renderDistanceLimiter();
		draw(render, 0, 0);
	}
}
