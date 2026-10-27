package com.vit.dungeon;

import java.awt.Canvas;
import java.awt.Color;
import java.awt.Cursor;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Point;
import java.awt.Toolkit;
import java.awt.image.BufferStrategy;
import java.awt.image.BufferedImage;
import java.awt.image.DataBufferInt;

import javax.swing.JFrame;

import com.vit.dungeon.graphics.Screen;
import com.vit.dungeon.input.Controller;
import com.vit.dungeon.input.InputHandler;
import com.vit.dungeon.input.MouseController;

public class Display extends Canvas implements Runnable {
	private static final long serialVersionUID = 1L;
	
	public static final int WIDTH = 800;
	public static final int HEIGHT = 600;
	public static final String TITLE = "Dungeon Pre-Alpha 0.0.2";
	
	private Thread thread;
	private BufferedImage img;
	private Game game;
	private Screen screen;
	private boolean running = false;
	private int[] pixels;
	private InputHandler input;
	private MouseController mouseController;
	
	private int fps;
	private int x;
	private int y;
	private int z;
	
	public static int MouseSpeed;
	
	
	private final Font debugFont = new Font("Hack", Font.PLAIN, 16);
	
	public Display() {
		Dimension size = new Dimension(WIDTH, HEIGHT);
		setPreferredSize(size);
		setMinimumSize(size);
		setMaximumSize(size);
	
		screen = new Screen(WIDTH, HEIGHT);
		game = new Game();
		img = new BufferedImage(WIDTH, HEIGHT, BufferedImage.TYPE_INT_RGB);
		pixels = ((DataBufferInt)img.getRaster().getDataBuffer()).getData();
		
		input = new InputHandler();
		addKeyListener(input);
		addFocusListener(input);
		addMouseListener(input);
		addMouseMotionListener(input);
	}
	
	private synchronized void start() {
		if (running) return;
		running = true;
		thread = new Thread(this, "GameThread");
		thread.start();
		System.out.println("Working");
	}
	
	public void run() {
		int frames = 0;
		double unprocessedSeconds = 0;
		long previousTime = System.nanoTime();
		double secondsPerTick = 1 / 60.00;
		int tickCount = 0;
		
		
		requestFocus();
		
		while (running) {
			long currentTime = System.nanoTime();
			long passedTime = currentTime - previousTime;
			previousTime = currentTime;
			unprocessedSeconds += passedTime / 1000000000.0;
			
			boolean ticked = false;
			while (unprocessedSeconds > secondsPerTick) {
				tick();
				unprocessedSeconds -= secondsPerTick;
				ticked = true;
				tickCount++;
				
				if (tickCount % 3 == 0) {
					x = (int) Controller.x / 16;
					y = (int) Controller.y / 16;
					z = (int) Controller.z / 16;
				}
				if (tickCount % 60 == 0) {
					fps = frames;
					frames = 0;
				}
			}
			
			// Renderizado único por vuelta si hubo un cambio lógico o para mantener fluidez
			render();
			frames++;
			
			if (input.key[java.awt.event.KeyEvent.VK_ESCAPE]) {
				mouseController.unlock();
			}

			if (mouseController.isLocked()) {
				mouseController.update();
			} else {
				Controller.turnLeft = input.key[java.awt.event.KeyEvent.VK_LEFT];
				Controller.turnRight = input.key[java.awt.event.KeyEvent.VK_RIGHT];
				MouseSpeed = 4;
			}
		}
	}
	
	private void tick() {
		game.tick(input.key);
	}
	
	private void render() {
		BufferStrategy bs = this.getBufferStrategy();
		if (bs == null) {
			createBufferStrategy(3); // Triple buffering óptimo
			return;
		}
		
		// 1. PRIMERO calculamos el renderizado 3D en la pantalla interna
		screen.render(game);
		
		// 2. DESPUÉS volcamos el array de píxeles procesados al búfer de la imagen
		System.arraycopy(screen.pixels, 0, pixels, 0, pixels.length);
		
		// 3. Dibujamos los gráficos e interfaz
		Graphics g = bs.getDrawGraphics();
		g.drawImage(img, 0, 0, WIDTH, HEIGHT, null);
		
		g.setFont(debugFont);
		g.setColor(Color.CYAN);
		g.drawString(fps + " FPS", 24, 32);
		
		String blockCoordinates = "Block: " + x + " " + y + " " + z;
		g.drawString(blockCoordinates, WIDTH - (blockCoordinates.length() * 10 + 24), 32);
		
		g.dispose();
		bs.show();
	}
	
	public static void main(String[] args) {
		BufferedImage cursor = new BufferedImage(16, 16, BufferedImage.TYPE_INT_ARGB);
		Cursor blank = Toolkit.getDefaultToolkit().createCustomCursor(cursor, new Point(0, 0), "blank");
		Display game = new Display();
		JFrame frame = new JFrame();
		frame.add(game);
		frame.pack();
		frame.getContentPane().setCursor(blank);
		frame.setTitle(TITLE);
		frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
		frame.setLocationRelativeTo(null);
		frame.setResizable(false);
		frame.setVisible(true);
		
		game.mouseController = new MouseController(frame, game.game);
		game.addMouseListener(new java.awt.event.MouseAdapter() {
			@Override
			public void mousePressed(java.awt.event.MouseEvent e) {
				game.mouseController.lock();
			}
		});
		
		System.out.println("Running...");
		game.start();
	}
}
