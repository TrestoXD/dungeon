package com.vit.dungeon.input;

import java.awt.Robot;
import java.awt.Cursor;
import java.awt.Point;
import java.awt.Toolkit;
import java.awt.image.BufferedImage;
import javax.swing.JFrame;

import com.vit.dungeon.Display;
import com.vit.dungeon.Game;

public class MouseController {
    private Robot robot;
    private JFrame frame;
    private Game game;
    private Cursor blankCursor;
    private boolean isLocked = false;

    public MouseController(JFrame frame, Game game) {
        this.frame = frame;
        this.game = game;
        try {
            this.robot = new Robot();
            // Creamos el cursor invisible una sola vez para optimizar memoria
            BufferedImage img = new BufferedImage(16, 16, BufferedImage.TYPE_INT_ARGB);
            this.blankCursor = Toolkit.getDefaultToolkit().createCustomCursor(img, new Point(0, 0), "blank");
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public void lock() {
        if (frame == null || isLocked) return;
        isLocked = true;
        frame.getContentPane().setCursor(blankCursor);
        centerMouse();
    }

    public void unlock() {
        if (frame == null || !isLocked) return;
        isLocked = false;
        frame.getContentPane().setCursor(Cursor.getDefaultCursor());
        
        // Limpiamos los giros al liberar el ratón para que no se quede girando solo
        Controller.turnLeft = false;
        Controller.turnRight = false;
        Display.MouseSpeed = 0;
    }

    private void centerMouse() {
        if (frame == null || !frame.isShowing()) return;
        int centerX = frame.getLocationOnScreen().x + frame.getWidth() / 2;
        int centerY = frame.getLocationOnScreen().y + frame.getHeight() / 2;
        robot.mouseMove(centerX, centerY);
    }

    public void update() {
        if (!isLocked || frame == null || !frame.isShowing()) return;

        // 1. Conseguir la posición física del ratón en la pantalla
        int mouseX = java.awt.MouseInfo.getPointerInfo().getLocation().x;
        int mouseY = java.awt.MouseInfo.getPointerInfo().getLocation().y;

        // 2. Calcular el centro de tu ventana
        int centerX = frame.getLocationOnScreen().x + frame.getWidth() / 2;
        int centerY = frame.getLocationOnScreen().y + frame.getHeight() / 2;

        int deltaX = mouseX - centerX;
        int deltaY = mouseY - centerY;

        // 3. Aplicar directamente las rotaciones e inercias a tu controlador
        if (deltaX > 0) {
            Controller.turnRight = true;
            Controller.turnLeft = false;
        } else if (deltaX < 0) {
            Controller.turnLeft = true;
            Controller.turnRight = false;
        } else {
            Controller.turnLeft = false;
            Controller.turnRight = false;
        }

        Display.MouseSpeed = Math.abs(deltaX);
        game.controls.ylooka -= deltaY * 2.0; // Sensibilidad vertical

        // 4. Regresar el ratón al centro para el siguiente frame
        centerMouse();
    }

    public boolean isLocked() {
        return isLocked;
    }
}
