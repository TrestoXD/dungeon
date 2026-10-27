package com.vit.dungeon.graphics;

import com.vit.dungeon.Game;
import com.vit.dungeon.input.Controller;
import com.vit.dungeon.world.Block;
import com.vit.dungeon.world.World;

public class Render3D extends Render {

	public double[] zBuffer;
	public double[] zBufferWall;
	private double renderdistance = 5000;
	private int spriteSheetWidth = 16;
	private double forward, right, up, cosine, sine, walking, yLook;


	private final double JITTER_GRID = 4.0; 

	public Render3D(int width, int height) {
		super(width, height);
		zBuffer = new double[width * height];
		zBufferWall = new double[width];
	}

	public void floor(Game game) {
		java.util.Arrays.fill(zBufferWall, 0);

		double floorPosition = 8;
		double ceilingPosition = 40;
		forward = Controller.z;
		right = Controller.x;
		up = Controller.y;
		
		yLook = Controller.ylook;
		
		walking = 0;
		up += 16;
		double rotation = Controller.rotation;
		cosine = Math.cos(rotation);
		sine = Math.sin(rotation);

		if (Controller.walk) {
			if (Controller.crouchWalk) {
				walking = Math.sin(game.time / 6.0) * 0.2;
			} else if (Controller.sprintWalk) {
				walking = Math.sin(game.time / 4.5) * 2;
			} else {
				walking = Math.sin(game.time / 6.0) * 0.5;
			}
		}

		int halfHeight = height / 2;
		double halfHeightInv = 1.0 / height;
		double halfWidth = width / 2.0;


		for (int y = 0; y < height; y++) {
			double ceiling = ((y - halfHeight) - yLook) * halfHeightInv;
			if (ceiling == 0) ceiling = 0.0001;

			double z;
			if (ceiling < 0) {
				z = (ceilingPosition - up - walking) / -ceiling;
			} else {
				z = (floorPosition + up + walking) / ceiling;
			}

			int rowOffset = y * width;
			double depthBase = z * halfHeightInv;

			int brightness = (int) (renderdistance / z);
			if (brightness < 0) brightness = 0;
			else if (brightness > 255) brightness = 255;

			for (int x = 0; x < width; x++) {
				double depth = (x - halfWidth) * depthBase;
				double xx = depth * cosine + z * sine;
				double yy = z * cosine - depth * sine;
				
				int xPix = (int) (xx + right);
				int yPix = (int) (yy + forward);
				
				int pixelIndex = x + rowOffset;
				zBuffer[pixelIndex] = z;

				if (z > 500) {
					pixels[pixelIndex] = 0;
				} else {
					int colour = Texture.block.pixels[(xPix & 15) + (yPix & 15) * spriteSheetWidth];
					
					int r = ((colour >> 16) & 0xff) * brightness / 255;
					int g = ((colour >> 8) & 0xff) * brightness / 255;
					int b = (colour & 0xff) * brightness / 255;
					pixels[pixelIndex] = r << 16 | g << 8 | b;
				}
			}
		}

		World world = game.world;
		int mapWidth = world.width;
		int mapHeight = world.height;
		int ysize = 3; 
		
		for (int yBlock = 0; yBlock < ysize; yBlock++) {
			for (int xBlock = 0; xBlock < mapWidth; xBlock++) {
				for (int zBlock = 0; zBlock < mapHeight; zBlock++) {
					Block block = world.create(xBlock, zBlock);
					
					if (block != null && block.solid) {
						Block north = world.create(xBlock, zBlock - 1);
						Block south = world.create(xBlock, zBlock + 1);
						Block west  = world.create(xBlock - 1, zBlock);
						Block east  = world.create(xBlock + 1, zBlock);
						
						if (north != null && !north.solid) renderWall(xBlock, xBlock + 1, zBlock, zBlock, yBlock);
						if (south != null && !south.solid) renderWall(xBlock + 1, xBlock, zBlock + 1, zBlock + 1, yBlock);
						if (west  != null && !west.solid)  renderWall(xBlock, xBlock, zBlock + 1, zBlock, yBlock);
						if (east  != null && !east.solid)  renderWall(xBlock + 1, xBlock + 1, zBlock, zBlock + 1, yBlock);
					}
				}
			}
		}
	}

	public void renderWall(double xLeft, double xRight, double zDistanceLeft, double zDistanceRight, double yHeight) {
		double xcLeft = (xLeft - (right / 16)) * 2;
		double zcLeft = (zDistanceLeft - (forward / 16)) * 2;

		double rotLeftSideX = xcLeft * cosine - zcLeft * sine;
		double walkCorrection = -up / 16 + -(walking / 16);
		double yCornerTL = ((-0.5 - yHeight) - walkCorrection) * 2;
		double yCornerBL = ((+0.5 - yHeight) - walkCorrection) * 2;
		double rotLeftSideZ = zcLeft * cosine + xcLeft * sine;

		double xcRight = (xRight - (right / 16)) * 2;
		double zcRight = (zDistanceRight - (forward / 16)) * 2;

		double rotRightSideX = xcRight * cosine - zcRight * sine;
		double rotRightSideZ = zcRight * cosine + xcRight * sine;

		double clip = 0.1;
		if (rotLeftSideZ < clip && rotRightSideZ < clip) return;
		
		if (rotLeftSideZ < clip) {
			double clip0 = (clip - rotLeftSideZ) / (rotRightSideZ - rotLeftSideZ);
			rotLeftSideZ = rotLeftSideZ + (rotRightSideZ - rotLeftSideZ) * clip0;
			rotLeftSideX = rotLeftSideX + (rotRightSideX - rotLeftSideX) * clip0;
		}
		if (rotRightSideZ < clip) {
			double clip0 = (clip - rotLeftSideZ) / (rotRightSideZ - rotLeftSideZ);
			rotRightSideZ = rotLeftSideZ + (rotRightSideZ - rotLeftSideZ) * clip0;
			rotRightSideX = rotLeftSideX + (rotRightSideX - rotLeftSideX) * clip0;
		}

		double xPixelLeftRaw = (rotLeftSideX / rotLeftSideZ * height + width / 2.0);
		double xPixelRightRaw = (rotRightSideX / rotRightSideZ * height + width / 2.0);
		
		double xPixelLeft = Math.floor(xPixelLeftRaw / JITTER_GRID) * JITTER_GRID;
		double xPixelRight = Math.floor(xPixelRightRaw / JITTER_GRID) * JITTER_GRID;
		
		if (xPixelLeft >= xPixelRight) return;

		int xPixelLeftInt = Math.max(0, (int) xPixelLeft);
		int xPixelRightInt = Math.min(width, (int) xPixelRight);

		double yPixelLeftTop = Math.floor((yCornerTL / rotLeftSideZ * height + height / 2.0 + yLook) / JITTER_GRID) * JITTER_GRID;
		double yPixelRightTop = Math.floor((yCornerTL / rotRightSideZ * height + height / 2.0 + yLook) / JITTER_GRID) * JITTER_GRID;
		double yPixelLeftBottom = Math.floor((yCornerBL / rotLeftSideZ * height + height / 2.0 + yLook) / JITTER_GRID) * JITTER_GRID;
		double yPixelRightBottom = Math.floor((yCornerBL / rotRightSideZ * height + height / 2.0 + yLook) / JITTER_GRID) * JITTER_GRID;

		double texStart = 0.0;
		double texEnd = 16.0;
		double xPixelDeltaInv = 1.0 / (xPixelRight - xPixelLeft);

		double zWallLeft = 1.0 / (1.0 / rotLeftSideZ) * 8;
		double zWallRight = 1.0 / (1.0 / rotRightSideZ) * 8;

		double zWallBufferStart = 1.0 / rotLeftSideZ;
		double zWallBufferDelta = (1.0 / rotRightSideZ) - zWallBufferStart;

		for (int x = xPixelLeftInt; x < xPixelRightInt; x++) {
			double pixelRotation = (x - xPixelLeft) * xPixelDeltaInv;

			double zWallBufferCheck = zWallBufferStart + zWallBufferDelta * pixelRotation;
			if (zBufferWall[x] > zWallBufferCheck) continue;
			zBufferWall[x] = zWallBufferCheck;

			int xTexture = (int) (texStart + (texEnd - texStart) * pixelRotation);

			double yTop = yPixelLeftTop + (yPixelRightTop - yPixelLeftTop) * pixelRotation;
			double yBottom = yPixelLeftBottom + (yPixelRightBottom - yPixelLeftBottom) * pixelRotation;

			int yPixelTopInt = Math.max(0, (int) yTop);
			int yPixelBottomInt = Math.min(height, (int) yBottom);
			
			double yPixelDeltaInv = 1.0 / (yBottom - yTop);
			double zCalculated = zWallLeft + (zWallRight - zWallLeft) * pixelRotation;

			int brightness = (int) (renderdistance / zCalculated);
			if (brightness < 0) brightness = 0;
			else if (brightness > 255) brightness = 255;

			for (int y = yPixelTopInt; y < yPixelBottomInt; y++) {
				double pixelRotationY = (y - yTop) * yPixelDeltaInv;
				int yTexture = (int) (16 * pixelRotationY); 
				
				int colour = Texture.stone.pixels[(xTexture & 15) + (yTexture & 15) * 16];
				
				int r = ((colour >> 16) & 0xff) * brightness / 255;
				int g = ((colour >> 8) & 0xff) * brightness / 255;
				int b = (colour & 0xff) * brightness / 255;

				int pixelIndex = x + y * width;
				pixels[pixelIndex] = r << 16 | g << 8 | b;
				zBuffer[pixelIndex] = zCalculated;
			}
		}
	}

	public void renderBlock(int xbpos, int ybpos, int zbpos) {
		renderWall(xbpos + 1, xbpos + 1, zbpos, zbpos + 1, ybpos);
		renderWall(xbpos, xbpos + 1, zbpos, zbpos, ybpos);
		renderWall(xbpos, xbpos, zbpos+1, zbpos, ybpos);
		renderWall(xbpos + 1, xbpos, zbpos+1, zbpos+1 , ybpos);
	}
	
	public void renderDistanceLimiter() {
		
	}
}
