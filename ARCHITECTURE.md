Brief overview of the project's architecture.

3 major components:
1. Canvas - lightweight abstraction over SDL2's `Canvas` that supports writing to a given pixel position.
2. Scene - Contains the geometry, BVH. Has some helpers in world space.
3. Raytracer - Handles "raytracing logic" as well as transforms from world -> screen space.


The `Raytracer` queries the `Scene` and then writes to the `Canvas`. Most of the parallelism is in the `Raytracer`, with the `Scene` assumed to be static while a render is in progress.
