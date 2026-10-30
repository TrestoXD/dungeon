package com.vit.dungeon.graphics;

import java.util.Arrays;

import com.vit.dungeon.Game;
import com.vit.dungeon.input.Controller;
import com.vit.dungeon.world.Block;
import com.vit.dungeon.world.World;

public class Render3D extends Render {

	private static final float FLOOR_POS = 8f;
	private static final float CEILING_POS = 40f;
	private static final float Z_CUTOFF = 500f;
	private static final float REF_HEIGHT = 600f;   
	private static final float CLIP = 0.1f;
	private static final int WALL_LEVELS = 3;        

	/** Rejilla de ajuste de vertices: 1 = pixel entero (PSX puro, con temblor). 2 o 4 = mas suave. */
	private static final int SNAP = 1;

	private static final int FIX_SHIFT = 16;
	private static final float FIX_ONE = 1 << FIX_SHIFT;
	private static final int MAX_LEVEL = Texture.FOG_LEVELS - 1;

	static final byte AIR = 0, SOLID = 1, VOID = 2;

	private final Lighting lighting = new Lighting();

	private final float[] zBufferWall;

	// Shifts for Textures
	private final int blockMaskU, blockMaskV, blockShiftV, blockTileShift;
	private final int stoneMaskU, stoneMaskV, stoneShiftV, stoneTileShift;
	private final int stoneW, stoneH;

	// State of the camera per frame
	private float forward, right, up, cosine, sine, walking, yLook;
	private float camX, camZ, camYCorrection, halfW, halfH;

	private byte[] grid = new byte[0];
	private int gridStride, mapW, mapH, gridHash;

	public Render3D(int width, int height, int[] target) {
		super(width, height, target);
		zBufferWall = new float[width];

		blockMaskU = Texture.block.width - 1;
		blockMaskV = Texture.block.height - 1;
		blockShiftV = Texture.log2(Texture.block.width);
		blockTileShift = Texture.tileShift(Texture.block);

		stoneW = Texture.stone.width;
		stoneH = Texture.stone.height;
		stoneMaskU = stoneW - 1;
		stoneMaskV = stoneH - 1;
		stoneShiftV = Texture.log2(stoneW);
		stoneTileShift = Texture.tileShift(Texture.stone);
	}

	public Lighting getLighting() {
		return lighting;
	}

	public void render(Game game) {
		setupCamera(game);
		Arrays.fill(zBufferWall, 0f);
		boolean worldChanged = buildGrid(game.world);
		lighting.update(right / 16f, forward / 16f, grid, gridStride, mapW, mapH, worldChanged, game.time);
		renderFloorAndCeiling();
		renderWalls();
	}

	private void setupCamera(Game game) {
		forward = (float) Controller.z;
		right = (float) Controller.x;
		up = (float) Controller.y + 16f;
		yLook = (float) Controller.ylook * height / REF_HEIGHT;

		double rotation = Controller.rotation;
		cosine = (float) Math.cos(rotation);
		sine = (float) Math.sin(rotation);

		walking = 0f;
		if (Controller.walk) {
			if (Controller.crouchWalk) {
				walking = (float) (Math.sin(game.time / 6.0) * 0.2);
			} else if (Controller.sprintWalk) {
				walking = (float) (Math.sin(game.time / 4.5) * 2.0);
			} else {
				walking = (float) (Math.sin(game.time / 6.0) * 0.5);
			}
		}

		camX = right / 16f;
		camZ = forward / 16f;
		camYCorrection = -(up + walking) / 16f;
		halfW = width * 0.5f;
		halfH = height * 0.5f;
	}

	private void renderFloorAndCeiling() {
		final int halfHeight = height / 2;
		final float invHeight = 1f / height;
		final float halfWidth = width * 0.5f;

		final int[] tex = Texture.blockFlat;
		final int shift = blockTileShift;
		final int mu = blockMaskU, mv = blockMaskV, sv = blockShiftV;

		final int[] lv = lighting.vertexLight;
		final int ls = lighting.stride;
		final int cw = mapW, ch = mapH;
		final int outside = (lighting.ambientFix() >> Lighting.FIX_SHIFT) << shift; // Out of the map

		for (int y = 0; y < height; y++) {
			float c = ((y - halfHeight) - yLook) * invHeight;
			if (c == 0f) c = 0.0001f;

			float z = (c < 0f)
					? (CEILING_POS - up - walking) / -c
					: (FLOOR_POS + up + walking) / c;

			int rowOffset = y * width;

			if (!(z > 0f && z <= Z_CUTOFF)) {
				Arrays.fill(pixels, rowOffset, rowOffset + width, 0);
				continue;
			}

			float depthBase = z * invHeight;
			float depth0 = -halfWidth * depthBase;
			int u = (int) ((depth0 * cosine + z * sine + right) * FIX_ONE);
			int v = (int) ((z * cosine - depth0 * sine + forward) * FIX_ONE);
			int du = (int) (depthBase * cosine * FIX_ONE);
			int dv = (int) (-depthBase * sine * FIX_ONE);

			int idx = rowOffset;
			final int end = rowOffset + width;
			while (idx < end) {
				int cx = u >> 20;
				int cz = v >> 20;
				int base;
				if (cx >= 0 && cx < cw && cz >= 0 && cz < ch) {
					int i = cx + cz * ls;
					int fx = (u >> 12) & 255;
					int fz = (v >> 12) & 255;
					int a = lv[i], b = lv[i + 1], cc = lv[i + ls], d = lv[i + ls + 1];
					int top = a + (((b - a) * fx) >> 8);
					int bot = cc + (((d - cc) * fx) >> 8);
					base = ((top + (((bot - top) * fz) >> 8)) >> Lighting.FIX_SHIFT) << shift;
				} else {
					base = outside;
				}
				pixels[idx++] = tex[base + ((u >> FIX_SHIFT) & mu) + (((v >> FIX_SHIFT) & mv) << sv)];
				u += du;
				v += dv;
			}
		}
	}

	/**
	 * Una sola consulta al mundo por celda. Devuelve true si el contenido del mundo cambio
	 * respecto al frame anterior (para que Lighting recalcule las sombras de la antorcha).
	 */
	private boolean buildGrid(World world) {
		int w = world.width, h = world.height;
		mapW = w;
		mapH = h;
		gridStride = w + 2;
		int needed = gridStride * (h + 2);
		if (grid.length != needed) grid = new byte[needed];

		int hash = 17;
		for (int z = -1; z <= h; z++) {
			for (int x = -1; x <= w; x++) {
				Block b = world.create(x, z);
				byte cell = (b == null) ? VOID : (b.solid ? SOLID : AIR);
				grid[(x + 1) + (z + 1) * gridStride] = cell;
				hash = hash * 31 + cell;
			}
		}
		boolean changed = hash != gridHash;
		gridHash = hash;
		return changed;
	}

	private void renderWalls() {
		final int w = mapW, h = mapH;
		final int s = gridStride;

		for (int z = 0; z < h; z++) {
			for (int x = 0; x < w; x++) {
				int i = (x + 1) + (z + 1) * s;
				if (grid[i] != SOLID) continue;

				if (grid[i - s] == AIR) renderWall(x, z, x + 1, z);         // norte
				if (grid[i + s] == AIR) renderWall(x + 1, z + 1, x, z + 1); // sur
				if (grid[i - 1] == AIR) renderWall(x, z + 1, x, z);         // oeste
				if (grid[i + 1] == AIR) renderWall(x + 1, z, x + 1, z + 1); // este
			}
		}
	}

	/** Dibuja una cara de la celda, de (xl, zl) a (xr, zr), en coordenadas de rejilla (enteras). */
	private void renderWall(int xl0, int zl0, int xr0, int zr0) {
		final int ls = lighting.stride;
		final int[] lv = lighting.vertexLight;
		final float lightA = lv[xl0 + zl0 * ls];
		final float lightB = lv[xr0 + zr0 * ls];

		float xcL = (xl0 - camX) * 2f;
		float zcL = (zl0 - camZ) * 2f;
		float xcR = (xr0 - camX) * 2f;
		float zcR = (zr0 - camZ) * 2f;

		float lx = xcL * cosine - zcL * sine;
		float lz = zcL * cosine + xcL * sine;
		float rx = xcR * cosine - zcR * sine;
		float rz = zcR * cosine + xcR * sine;

		if (lz < CLIP && rz < CLIP) return;

		float tL = 0f, tR = 1f;
		if (lz < CLIP) {
			float t = (CLIP - lz) / (rz - lz);
			lx += (rx - lx) * t;
			lz = CLIP;
			tL = t;
		} else if (rz < CLIP) {
			float t = (CLIP - lz) / (rz - lz);
			rx = lx + (rx - lx) * t;
			rz = CLIP;
			tR = t;
		}

		final float snap = SNAP;
		float xl = (float) Math.floor((lx / lz * height + halfW) * snap) / snap;
		float xr = (float) Math.floor((rx / rz * height + halfW) * snap) / snap;
		if (xl >= xr) return; 

		int x0 = Math.max(0, (int) Math.ceil(xl));
		int x1 = Math.min(width, (int) Math.ceil(xr));
		if (x0 >= x1) return;

		float yTopCorner = ((-0.5f - (WALL_LEVELS - 1)) - camYCorrection) * 2f;
		float yBotCorner = (0.5f - camYCorrection) * 2f;

		float invLz = 1f / lz;
		float invRz = 1f / rz;
		float yLookOff = halfH + yLook;

		float tlY = (float) Math.floor((yTopCorner * invLz * height + yLookOff) * snap) / snap;
		float trY = (float) Math.floor((yTopCorner * invRz * height + yLookOff) * snap) / snap;
		float blY = (float) Math.floor((yBotCorner * invLz * height + yLookOff) * snap) / snap;
		float brY = (float) Math.floor((yBotCorner * invRz * height + yLookOff) * snap) / snap;

		final float texW = stoneW;
		final float texH = stoneH * WALL_LEVELS;
		final float uL = tL * texW;
		final float dU = (tR - tL) * texW;
		final float lightL = lightA + (lightB - lightA) * tL;
		final float dLight = (lightB - lightA) * (tR - tL);

		float invDx = 1f / (xr - xl);
		float dInvZ = invRz - invLz;

		final int[] tex = Texture.stoneFlat;
		final int shift = stoneTileShift;
		final int mu = stoneMaskU, mv = stoneMaskV, sv = stoneShiftV;

		for (int x = x0; x < x1; x++) {
			float t = (x - xl) * invDx;

			float invZ = invLz + dInvZ * t;
			if (zBufferWall[x] > invZ) continue;
			zBufferWall[x] = invZ;

			float yTop = tlY + (trY - tlY) * t;
			float yBot = blY + (brY - blY) * t;
			if (yBot <= yTop) continue;

			int yStart = yTop < 0f ? 0 : (int) yTop;
			int yEnd = yBot > height ? height : (int) yBot;
			if (yStart >= yEnd) continue;

			int level = (int) (lightL + dLight * t) >> Lighting.FIX_SHIFT;
			if (level > MAX_LEVEL) level = MAX_LEVEL;
			int base = (level << shift) + ((int) (uL + dU * t) & mu);

			float vStep = texH / (yBot - yTop);
			int dv = (int) (vStep * FIX_ONE);
			int v = (int) ((yStart - yTop) * vStep * FIX_ONE);
			if (v < 0) v = 0;

			int idx = yStart * width + x;
			for (int y = yStart; y < yEnd; y++) {
				pixels[idx] = tex[base + (((v >> FIX_SHIFT) & mv) << sv)];
				idx += width;
				v += dv;
			}
		}
	}
}