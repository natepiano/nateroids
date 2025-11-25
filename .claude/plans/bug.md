i have a crazy situation that i cannot figure out.  i have a camera with bloom that renders on render layer 0 and order 0.  And a camerea without bloom that rnders on render layer 1 and order 1.  And it's all working fine - the order 0 camera is showing bloom-y things and order 1 is showing non-bloom-y things.
t
The problem is that for some reason that i cannot yet isolate - if I insert a random component - even on some nother entity (such as the primary window), then on launch, I see the render from the bloom camera flash for say, 1 frame, and then it disappears and  i only see the rendering from the non-bloom camera.

If i comment out the component insert and re-launch - bloom appears.  So I seemingly am always one component insert away from inadvertently disabling the camera.
