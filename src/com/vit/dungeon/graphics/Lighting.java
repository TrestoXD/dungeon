package com.vit.dungeon.graphics;

public class Lighting {
	public static final int FIX_SHIFT = 8;
	private static final int MAX_FIX = (Texture.FOG_LEVELS - 1) << FIX_SHIFT;
	private static final float LOS_STEP = 0.25f;

	public float ambient = 1.5f;
	public float radius = 10f;
	public float intensity = 30f;
	public float flickerAmount = 1f;

	public int[] vertexLight = new int[0];
	public int stride;

	private int vw, vh; 

	private byte[] grid;
	private int gridStride;

	private int lastStep = Integer.MIN_VALUE;
	private float lastPx = Float.NaN, lastPz = Float.NaN;
	private float lastRadius = Float.NaN, lastIntensity = Float.NaN, lastAmbient = Float.NaN;

	public int ambientFix() {
		int a = (int) (ambient * (1 << FIX_SHIFT));
		return a < 0 ? 0 : (a > MAX_FIX ? MAX_FIX : a);
	}

	void update(float px, float pz, byte[] grid, int gridStride, int w, int h, boolean worldChanged, int time) {
		this.grid = grid;
		this.gridStride = gridStride;

		int nvw = w + 1, nvh = h + 1;
		if (nvw != vw || nvh != vh) {
			vw = nvw;
			vh = nvh;
			stride = vw;
			vertexLight = new int[vw * vh];
			worldChanged = true;
		}

		int step = time >> 2;
		boolean unchanged = !worldChanged
				&& step == lastStep
				&& px == lastPx && pz == lastPz
				&& radius == lastRadius && intensity == lastIntensity && ambient == lastAmbient;
		if (unchanged) return; 

		lastStep = step;
		lastPx = px;
		lastPz = pz;
		lastRadius = radius;
		lastIntensity = intensity;
		lastAmbient = ambient;

		int amb = ambientFix();
		java.util.Arrays.fill(vertexLight, amb);

		float peak = intensity * flicker(step);
		int x0 = Math.max(0, (int) Math.floor(px - radius));
		int x1 = Math.min(vw - 1, (int) Math.ceil(px + radius));
		int z0 = Math.max(0, (int) Math.floor(pz - radius));
		int z1 = Math.min(vh - 1, (int) Math.ceil(pz + radius));
		for (int vz = z0; vz <= z1; vz++) {
			int row = vz * vw;
			for (int vx = x0; vx <= x1; vx++) {
				float l = lightAt(vx, vz, px, pz, peak);
				if (l > 0f) {
					int v = amb + (int) (l * (1 << FIX_SHIFT));
					vertexLight[row + vx] = v > MAX_FIX ? MAX_FIX : v;
				}
			}
		}
	}

	private float flicker(int step) {
		if (flickerAmount <= 0f) return 1f;
		float t = step * 0.9f;
		return 1f + flickerAmount * (0.06f * (float) Math.sin(t) + 0.04f * (float) Math.sin(t * 2.7f + 1.3f));
	}

	private float lightAt(int vx, int vz, float lx, float lz, float peak) {
		float dx = vx - lx, dz = vz - lz;
		float d2 = dx * dx + dz * dz;
		if (d2 >= radius * radius) return 0f;
		float dist = (float) Math.sqrt(d2);
		if (blocked(lx, lz, dx, dz, dist)) return 0f;
		float s = 1f - dist / radius;
		return peak * s * s;
	}

	private boolean blocked(float lx, float lz, float dx, float dz, float dist) {
		float limit = dist - 0.05f;
		if (limit <= LOS_STEP) return false;
		float inv = LOS_STEP / dist;
		float sx = dx * inv, sz = dz * inv;
		float px = lx, pz = lz;
		for (float t = LOS_STEP; t < limit; t += LOS_STEP) {
			px += sx;
			pz += sz;
			int cx = (int) px;
			if (px < cx) cx--;
			int cz = (int) pz;
			if (pz < cz) cz--;
			if (solidAt(cx, cz)) return true;
		}
		return false;
	}

	private boolean solidAt(int cx, int cz) {
		if (cx < -1 || cz < -1 || cx > vw - 1 || cz > vh - 1) return true;
		return grid[(cx + 1) + (cz + 1) * gridStride] != Render3D.AIR;
	}
}