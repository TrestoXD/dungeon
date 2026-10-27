package com.vit.dungeon.engine;

import com.vit.dungeon.engine.graph.Render;
import com.vit.dungeon.engine.scene.Scene;

public interface IAppLogic {
	void cleanup();
	
	void init(Window window, Scene scene, Render render);
	
	void input(Window window, Scene scene, long diffTimeMillis);
	
	void update(Window window, Scene scene,long diffTimeMillis);
}
