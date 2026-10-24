package com.vit.dungeon;

import java.awt.event.KeyEvent;

import com.vit.dungeon.input.Controller;

public class Game {
	public int time;
	public Controller controls;
	
	public Game() {
		controls = new Controller(); 
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
		
		controls.tick(fordward, back, left, right, jump, crouch, sprint);
	}
}
