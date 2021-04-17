# Spaces to look at
- We have our element space (ES) in which our elements live in
- We have our sample space (SS) in which we took our samples
- Our aim is to interpolate our samples (which are elements) to other elements, such that:
  - points in our sample space (which weren't samples) are identified with elements ("inter")
  - the changes are smooth, that is, if we change our point in SS only slightly, our identified element (in ES) should also only change slightly

# Curves
- Linear expects a SS isomorph to R^1. The knots tell us, where we got our samples. We interpolate our elements linear depending on the distance to the knots.
- Bezier Curves are like Linear with only 2 samples with knots 0.0 and 1.0. Because there is no distance to measure, as we only have one "samplegroup", Bezier Curves don't need any knots.
- BSplines are the conglomeration of Bezier Curves added together with some smoothness criterium. Theses are like Linear with any amount of samples. Such, they also need knots. The amount of knots depend on how many "samplegroups" are there, which in turn is decided by the amount of samples and how smooth it should be (smoother -> more interlapping samples between samplegroups).
