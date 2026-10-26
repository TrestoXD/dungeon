package com.vit.dungeon.world;

public class Block {

	public boolean solid = false;
	
	public static Block solidWall = new SolidBlock();
	public static Block nonsolidWallBlock = new NonSolidWall();
}
