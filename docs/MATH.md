# Wolfenstein-3D Raycasting Math

## 1. Coordinate System

The world is a 2D grid of integer cells. The player lives in continuous floating-point space, so position $(p_x, p_y)$ sits inside cell $(\lfloor p_x \rfloor, \lfloor p_y \rfloor)$.

## 2. Player Direction

The player faces direction angle $\theta$ (radians). The unit direction vector is:

$$\vec{d} = (\cos\theta,\ \sin\theta)$$

Turning left/right changes $\theta$:

$$\theta \leftarrow \theta \pm \omega \Delta t$$

Moving forward/backward:

$$p_x \leftarrow p_x + v\cos\theta, \quad p_y \leftarrow p_y + v\sin\theta$$

## 3. Field of View

The horizontal FOV spans $60°\ (\pi/3\ \text{rad})$. For column $i$ out of $W$ total columns the ray angle is:

$$\theta_i = \theta - \frac{\text{FOV}}{2} + \frac{i}{W}\cdot\text{FOV}$$

## 4. DDA – Digital Differential Analyzer

Given ray direction $\vec{d} = (d_x, d_y)$ from position $(p_x, p_y)$:

**Step deltas** — distance the ray travels to cross one full grid unit:

$$\delta_x = \left|\frac{1}{d_x}\right|, \quad \delta_y = \left|\frac{1}{d_y}\right|$$

**Initial side distances** — distance to the very first grid line:

$$
s_x = \begin{cases}
(p_x - \lfloor p_x \rfloor)\,\delta_x & d_x < 0 \\
(\lfloor p_x \rfloor + 1 - p_x)\,\delta_x & d_x \geq 0
\end{cases}
$$

(Same formula for $s_y$.)

**Stepping loop** — at each iteration choose the axis with the smaller accumulated distance, step one cell on that axis, and check whether the new cell is solid:

```
loop:
  if s_x < s_y:
    s_x += δ_x;  cell_x += sign(d_x);  hit_side = VERTICAL
  else:
    s_y += δ_y;  cell_y += sign(d_y);  hit_side = HORIZONTAL
  if map[cell_y][cell_x] != EMPTY: break
```

## 5. Perpendicular Wall Distance

To avoid the fish-eye effect we use the **perpendicular** distance (projected onto the camera plane), not the Euclidean distance:

$$
d_\perp = \begin{cases}
s_x - \delta_x & \text{vertical hit} \\
s_y - \delta_y & \text{horizontal hit}
\end{cases}
$$

This equals the true distance only when the ray is perpendicular to the wall; for other angles it correctly de-fishes the result.

## 6. Wall Column Height

The projected height of a wall column on the screen uses similar triangles. If the wall cell is $1$ unit tall and the camera plane has height equal to the screen height $H$:

$$h = \frac{H}{d_\perp}$$

The column is drawn from $(H/2 - h/2)$ to $(H/2 + h/2)$ in screen-space pixels.

## 7. Texture / Wall-X Coordinate

The fractional position where the ray hit the wall face (for tinting):

$$
w_x = \begin{cases}
p_y + d_\perp \cdot d_y & \text{vertical hit} \\
p_x + d_\perp \cdot d_x & \text{horizontal hit}
\end{cases}
\quad \bmod 1
$$

## 8. Shading

Horizontal-side walls are $40\%$ darker than vertical-side walls, giving a cheap directional shading that makes corners pop without any lighting calculation.

## 9. Minimap

The minimap is a direct 2D projection of the grid — each cell maps to a small rectangle in NDC (Normalized Device Coordinates). The player dot and direction arrow are drawn on top.

NDC conversion for a pixel $(px, py)$ on a screen of size $(W, H)$:

$$x_{ndc} = \frac{2\,px}{W} - 1, \quad y_{ndc} = 1 - \frac{2\,py}{H}$$

## 10. Collision Detection

A simple AABB-style check: before moving in $X$ or $Y$, test whether the destination cell (with a small margin) is solid. If it is, block movement on that axis only, allowing sliding along walls.
