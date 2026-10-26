package com.vit.dungeon;

import java.awt.event.KeyEvent;

import com.vit.dungeon.input.Controller;
import com.vit.dungeon.world.World;

public class Game {
	public int time;
	public Controller controls;
	public World world;
	
	public Game() {
		controls = new Controller(); 
		world = new World(8, 8);
	}

	public void tick(boolean[] key) {
		time++;
		boolean fordward = key[KeyEvent.VK_W];
		boolean back = key[KeyEvent.VK_S];
		boolean left = key[KeyEvent.VK_A];
		boolean right = key[KeyEvent.VK_D];
		boolean jump = key[KeyEvent.VK_SPACE];
		boolean crouch = key[KeyEvent.VK_SHIFT];
		boolean sprint = key[KeyEvent.VK_CONTROL];
		boolean esc = key[KeyEvent.VK_ESCAPE];
		
		controls.tick(fordward, back, left, right, jump, crouch, sprint,esc, world);
		try {
			controls.freezeMouse();
		} catch (Exception e) {
			// TODO Auto-generated catch block
			e.printStackTrace();
		}
	}
}
