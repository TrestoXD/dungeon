package com.vit.dungeon.graphics;

import com.vit.dungeon.Game;

public class Screen extends Render {
	private final Render3D render3D;

	public Screen(int width, int height, int[] target) {
		super(width, height, target);
		render3D = new Render3D(width, height, target);
	}

	public Lighting getLighting() {
		return render3D.getLighting();
	}

	public void render(Game game) {
		render3D.render(game);
	}
}