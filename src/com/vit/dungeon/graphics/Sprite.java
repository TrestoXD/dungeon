package com.vit.dungeon.graphics;

public class Sprite {
    public double x, z, y; 
    public int textureIndex; 

    public Sprite(double x, double z, double y, int textureIndex) {
        this.x = x * 16.0 + 8.0;
        this.z = z * 16.0 + 8.0;
        this.y = y;
        this.textureIndex = textureIndex;
    }
}