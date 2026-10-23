package com.vit.dungeon.input;

public class Controller {
	public double x, z, rotation, ylook, xa, za, rotationa, ylooka;
	public static boolean turnLeft = false;
	public static boolean turnRight = false;
	
	public void tick(boolean forward, boolean back, boolean left, boolean right) {
		double rotationSpeed = 1.0;
		double walkSpeed = 1.0;
		double xMove = 0.0;
		double zMove = 0.0;
		
		if(forward) zMove++;
		if(back) zMove--;
		if(left) xMove--;
		if(right) xMove++;
		
		if(turnLeft) {
			rotationa -= rotationSpeed;
		}
		if(turnRight) {
			rotationa += rotationSpeed;
		}
		
		xa += (xMove * Math.cos(rotation) + zMove * Math.sin(rotation) * walkSpeed);
		za += (zMove * Math.cos(rotation) - xMove * Math.sin(rotation) * walkSpeed);
	
		x += xa;
		z += za;
		xa *= 0.1;
		za *= 0.1;
		
		rotation += rotationa;
		rotation *= 0.05;
		
		ylook += ylooka;
		ylooka *= 0.7;
		
		if (ylook > 360) ylook = 360;
		if (ylook < -300) ylook = -300;
	}
}
