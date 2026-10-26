package com.vit.dungeon.input;

import java.awt.Robot;
import java.awt.MouseInfo;

import com.vit.dungeon.Display;

public class Controller {
	public double x, y, z, rotation, ylook, xa, za, rotationa, ylooka;
	public static boolean turnLeft = false;
	public static boolean turnRight = false;
	public static boolean walk, sprintWalk, crouchWalk = false;
	private static boolean isMouseFrozen = false;
	
	public void tick(boolean forward, boolean back, boolean left, boolean right, boolean jump, boolean crouch, boolean sprint, boolean escape) {
		double rotationSpeed = 0.004 * Display.MouseSpeed;
		double walkSpeed = 0.5;
		double jumpHeight = 0.8;
		double crouchHeight = 0.3;
		double xMove = 0;
		double zMove = 0;
		
		if(forward) {
			zMove++;
			walk = true;
		}
		if(back) {
			zMove--;
			walk = true;
		}
		if(left) {
			xMove--;
			walk = true;
		}
		if(right) {
			xMove++;
			walk = true;
		}
		
		if(turnLeft) {
			rotationa -= rotationSpeed;
		}
		if(turnRight) {
			rotationa += rotationSpeed;
		}
		
		if(jump) {
			y += jumpHeight;
			sprint = false;
		}
		if(crouch) {
			walkSpeed = 0.2;
			y -= crouchHeight;
			sprint = false;
			crouchWalk = true;
		}
		if(sprint) {
			walkSpeed = 1.25;
			sprintWalk = true;
		}
		
		if(!forward && !back && !left && !right) {
			walk = false;
		}
		if(!crouch) {
			crouchWalk = false;
		}
		if(!sprint) {
			sprintWalk = false;
		}
		if(escape) {
			// BLOQUEA EL MOUSE
		}
		
		xa += (xMove * Math.cos(rotation) + zMove * Math.sin(rotation)) * walkSpeed;
		za += (zMove * Math.cos(rotation) - xMove * Math.sin(rotation)) * walkSpeed;
	
		x += xa;
		y *= 0.9;
		z += za;
		xa *= 0.1;
		za *= 0.1;
		rotation += rotationa;
		rotationa *= 0.5;
		
		ylook += ylooka;
		ylooka *= 0.7;
		
		if (ylook > 360) ylook = 360;
		if (ylook < -300) ylook = -300;
	}

	public void freezeMouse() throws Exception{
		Robot robot = new Robot();
		while(isMouseFrozen) {
			int x = MouseInfo.getPointerInfo().getLocation().x;
			int y = MouseInfo.getPointerInfo().getLocation().y;
			robot.mouseMove(x, y);
			Thread.sleep(10);
		}
	}
	
	public static void toggleMouseFreeze() {
		isMouseFrozen = !isMouseFrozen;
	}
}
