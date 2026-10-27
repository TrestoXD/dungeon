package com.vit.dungeon.world;

public class Block {

	public boolean solid = false;
	
	// Constructor por defecto
	public Block() {
	}

	public Block(boolean solid) {
		this.solid = solid;
	}
	
	public static Block solidWall = new SolidBlock();
	public static Block nonsolidWallBlock = new NonSolidWall();
}
