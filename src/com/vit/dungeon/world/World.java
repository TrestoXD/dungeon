package com.vit.dungeon.world;

import java.util.Random;

import com.vit.dungeon.input.Controller;

public class World {
	private Block[][] blocks;
	public int width;
	public int height;

	public World(int width, int height) {
		this.width = width;
		this.height = height;
		this.blocks = new Block[width][height];
		
		generarMazmorraAleatoria();
	}

	private void generarMazmorraAleatoria() {
		Random random = new Random();

		for (int x = 0; x < width; x++) {
			for (int y = 0; y < height; y++) {
				blocks[x][y] = new Block(true); 
			}
		}

		int ex = width / 2;
		int ey = height / 2;
		
		blocks[ex][ey] = new Block(false); 

		int celdasAAbrir = (int) ((width * height) * 0.40);
		int celdasAbiertas = 0;
		int[][] direcciones = {{0, -1}, {0, 1}, {-1, 0}, {1, 0}};

		while (celdasAbiertas < celdasAAbrir) {
			int[] dir = direcciones[random.nextInt(4)];
			int nuevoX = ex + dir[0];
			int nuevoY = ey + dir[1];

			if (nuevoX > 0 && nuevoX < width - 1 && nuevoY > 0 && nuevoY < height - 1) {
				ex = nuevoX;
				ey = nuevoY;

				if (blocks[ex][ey].solid) {
					blocks[ex][ey] = new Block(false); 
					celdasAbiertas++;
				}
			} else {
				do {
					ex = random.nextInt(width - 2) + 1;
					ey = random.nextInt(height - 2) + 1;
				} while (blocks[ex][ey].solid);
			}
		}
		
		System.out.println("¡Mazmorra procedural generada! Pasadizos excavados: " + celdasAbiertas);


		boolean spawnEncontrado = false;
		
		for (int xBlock = width / 2; xBlock < width - 1; xBlock++) {
			for (int zBlock = height / 2; zBlock < height - 1; zBlock++) {
				if (!blocks[xBlock][zBlock].solid) {
					Controller.x = (xBlock * 16.0) + 8.0;
					Controller.z = (zBlock * 16.0) + 8.0;
					Controller.y = 0; 
					spawnEncontrado = true;
					break;
				}
			}
			if (spawnEncontrado) break;
		}


		if (!spawnEncontrado) {
			for (int xBlock = 1; xBlock < width - 1; xBlock++) {
				for (int zBlock = 1; zBlock < height - 1; zBlock++) {
					if (!blocks[xBlock][zBlock].solid) {
						Controller.x = (xBlock * 16.0) + 8.0;
						Controller.z = (zBlock * 16.0) + 8.0;
						spawnEncontrado = true;
						break;
					}
				}
				if (spawnEncontrado) break;
			}
		}
		
		System.out.println("Jugador posicionado de forma segura en las coordenadas: X=" + Controller.x + " Z=" + Controller.z);
	}


	public Block create(int x, int z) {
		if (x < 0 || x >= width || z < 0 || z >= height) {
			return Block.solidWall;
		}
		return blocks[x][z];
	}
	
	public boolean isSolidAt(double xCoord, double zCoord) {

		int xBlock = (int) Math.floor(xCoord / 16.0);
		int zBlock = (int) Math.floor(zCoord / 16.0);

		Block block = create(xBlock, zBlock);
		return block != null && block.solid;
	}
}
