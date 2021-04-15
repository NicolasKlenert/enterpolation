## Enterpolation is the most expressive and easy to use interpolation library out there.

# Language:
- Knots are points. These live in Euclidian Space, such having a distance between them (there should be nor reason to make it even more generalised)
- Elements are the elements to be interpolated. How to interpolate will be defined by the interpolation method. However (for know) these elements should work like Vectorspaces, that is, act linear and having addition as well as scalar multiplication. Scalars in our case are (single) components of knots.
- bezier and bspline surfaces NEED a grid to work! (but not a rigid grid) -> such knots can't be used without any constraints. However, building the interpolation from the bottom up, there is not a problem!
- bezier and bspline (and NURBS) are more curve fitting problems and not strictly interpolation problems, because the don't go through the points given (but still populate an area (inside of the convex hull of the points given))

# Thoughts:
- InterScalar does not have to be a scalar -> usually we just want to say: hey, we have n points and how much to they factor in in all that?
- 4 points to factor in can be because of: we have a simplex in 3-dim space OR we have a rectangle in 2-dim space OR we have 4 points in a line in 1-dim and they factor all in
- If we look at noise, it is more or less the same, it interpolates. However it tries to generate everything on the fly, including the elements. Such Interpolation does: look at specific knots (the algo often decides which one) -> generate elements there -> interpolate
- bezier and bspline do more like: look at specific knots (given beforehand) -> get elements there (given) -> interpolate

# Interpolation:
- consists of: Knots and the sampled/generated/given elements there -> interpolate with weights (usually...is there any other form? I do not think so)
- such knots are identified with elements. The knots give us information (most of the time distance) and are used to determine the weights.
- It IS reasonable to assume that we want f64 (or f32) as knot-components as we want to work with R^n (almost) ever and ALSO that we just use points as knots (we should think about how to make a nice API)


# Points:
- This is given for the get methods. I would like that all of these are possible: get(x,y), get([x,y]), get(Point).
- Because rust does not have function overloading, we would need a trait for get(x,y) just to have it.. This feels like an anti-pattern
- Such we could see [x,y] as a Point and just allow Points OR just allow [f64;N] and every Point should use .to_array() -> [f64;N]
