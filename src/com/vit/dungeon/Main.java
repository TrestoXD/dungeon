package com.vit.dungeon;

import com.vit.dungeon.engine.*;
import com.vit.dungeon.engine.graph.Mesh;
import com.vit.dungeon.engine.graph.Render;
import com.vit.dungeon.engine.scene.Scene;

import com.vit.dungeon.engine.Engine;
import com.vit.dungeon.engine.IAppLogic;
import com.vit.dungeon.engine.Window;

public class Main implements IAppLogic {

    public static void main(String[] args) {
        Main main = new Main();
        Engine gameEng = new Engine("chapter-03", new Window.WindowOptions(), main);
        gameEng.start();
    }

    @Override
    public void cleanup() {
        // Nothing to be done yet
    }

    @Override
    public void init(Window window, Scene scene, Render render) {
        float[] positions = new float[]{
                0.0f, 0.5f, 0.0f,
                -0.5f, -0.5f, 0.0f,
                0.5f, -0.5f, 0.0f
        };
        Mesh mesh = new Mesh(positions, 3);
        scene.addMesh("triangle", mesh);
    }

    @Override
    public void input(Window window, Scene scene, long diffTimeMillis) {
        // Nothing to be done yet
    }

    @Override
    public void update(Window window, Scene scene, long diffTimeMillis) {
        // Nothing to be done yet
    }
}