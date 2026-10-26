package com.vit.dungeon.world;

public class World {
	public Block[] blocks;
	public final int width;
	public final int height;
	
	public World(int width, int height) {
		this.width = width;
		this.height = height;
		blocks = new Block[width * height];
		int[][] publictestmap = {
				{1,1,1,1,1,1,1,1},
				{1,0,0,0,0,0,0,1},
				{1,0,0,0,0,0,0,1},
				{0,0,0,0,0,0,0,0},
				{0,0,0,0,0,0,0,0},
				{1,0,0,0,0,0,0,1},
				{1,0,0,0,0,0,0,1},
				{1,1,1,0,0,1,1,1}
				};
		for (int y = 0; y < height; y++) {
			for (int x = 0; x < width; x++) {
				Block block = null;
				if(publictestmap[y][x] == 1) {
					block = new SolidBlock();
				}else {
					block = new Block();
				}
				blocks[x + y * width] = block;
			}
		}
	}
	
	public Block create(int x, int y) {
		if(x < 0 || y < 0 || x >= width || y >= height) {
			return Block.solidWall;
		}
		return blocks[x + y * width];
	}
}
