package com.vit.dungeon;

import java.awt.Canvas;
import java.awt.Color;
import java.awt.Cursor;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Point;
import java.awt.Toolkit;
import java.awt.event.KeyEvent;
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

	public static final int RENDER_WIDTH = 320;
	public static final int RENDER_HEIGHT = 240;
	public static final int SCALE = 3;

	public static final int WIDTH = RENDER_WIDTH * SCALE;
	public static final int HEIGHT = RENDER_HEIGHT * SCALE;
	public static final String TITLE = "Dungeon Pre-Alpha 0.0.2";

	private static final long NANOS_PER_TICK = 1000000000L / 60;
	private static final long MAX_LAG = NANOS_PER_TICK * 5; 

	private Thread thread;
	private BufferedImage img;
	private BufferStrategy strategy;
	private Game game;
	private Screen screen;
	private volatile boolean running = false;
	private InputHandler input;
	private MouseController mouseController;

	private int fps;
	private boolean debugmode = false;
	private boolean f3WasDown = false;
	private String blocksDebug = "";

	public static int MouseSpeed;

	private final Font debugFont = new Font("Hack", Font.PLAIN, 16);

	public Display() {
		Dimension size = new Dimension(WIDTH, HEIGHT);
		setPreferredSize(size);
		setMinimumSize(size);
		setMaximumSize(size);
		setIgnoreRepaint(true); 

		img = new BufferedImage(RENDER_WIDTH, RENDER_HEIGHT, BufferedImage.TYPE_INT_RGB);
		int[] pixels = ((DataBufferInt) img.getRaster().getDataBuffer()).getData();
		screen = new Screen(RENDER_WIDTH, RENDER_HEIGHT, pixels);
		game = new Game();

		input = new InputHandler();
		addKeyListener(input);
		addFocusListener(input);
		addMouseListener(input);
		addMouseMotionListener(input);
	}

	private synchronized void start() {
		if (running) return;
		raiseTimerResolution();
		createBufferStrategy(2); 
		strategy = getBufferStrategy();
		running = true;
		thread = new Thread(this, "GameThread");
		thread.start();
		System.out.println("Working");
	}
	private static void raiseTimerResolution() {
		Thread t = new Thread("TimerResolution") {
			@Override
			public void run() {
				try {
					Thread.sleep(Long.MAX_VALUE);
				} catch (InterruptedException e) {
					// End
				}
			}
		};
		t.setDaemon(true);
		t.start();
	}

	public void run() {
		long previous = System.nanoTime();
		long lag = 0;
		long fpsTimer = previous;
		int frames = 0;
		int tickCount = 0;

		requestFocus();

		while (running) {
			long now = System.nanoTime();
			lag += now - previous;
			previous = now;
			if (lag > MAX_LAG) lag = MAX_LAG;

			handleInput();

			boolean ticked = false;
			while (lag >= NANOS_PER_TICK) {
				pollMouse();
				tick();
				lag -= NANOS_PER_TICK;
				ticked = true;
				tickCount++;

				if (debugmode && tickCount % 3 == 0) {
					blocksDebug = "Block: " + (int) Controller.x / 16 + " "
							+ (int) Controller.y / 16 + " " + (int) Controller.z / 16;
				}
			}

			if (ticked) {
				render();
				frames++;
				if (now - fpsTimer >= 1000000000L) {
					fps = frames;
					frames = 0;
					fpsTimer = now;
				}
			} else {
				long waitMs = (NANOS_PER_TICK - lag) / 1000000L;
				try {
					if (waitMs > 1) Thread.sleep(waitMs - 1);
					else Thread.yield();
				} catch (InterruptedException e) {
					Thread.currentThread().interrupt();
					running = false;
				}
			}
		}
	}

	private void handleInput() {
		if (input.key[KeyEvent.VK_ESCAPE] || input.key[KeyEvent.VK_WINDOWS]) {
			mouseController.unlock();
		}

		boolean f3 = input.key[KeyEvent.VK_F3];
		if (f3 && !f3WasDown) debugmode = !debugmode;
		f3WasDown = f3;

	}

	private void pollMouse() {
		if (mouseController.isLocked()) {
			mouseController.update();
		} else {
			Controller.turnLeft = input.key[KeyEvent.VK_LEFT];
			Controller.turnRight = input.key[KeyEvent.VK_RIGHT];
			MouseSpeed = 4;
		}
	}

	private void tick() {
		game.tick(input.key);
	}

	private void render() {
		screen.render(game);

		do {
			do {
				Graphics g = strategy.getDrawGraphics();
				g.drawImage(img, 0, 0, WIDTH, HEIGHT, null);

				if (debugmode) {
					g.setFont(debugFont);
					g.setColor(Color.CYAN);
					g.drawString(fps + " FPS", 24, 32);
					g.drawString(blocksDebug, WIDTH - (blocksDebug.length() * 10), 32);
				}
				g.dispose();
			} while (strategy.contentsRestored());
			strategy.show();
		} while (strategy.contentsLost());
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