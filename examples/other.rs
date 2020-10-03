use axgeom::{rect, vec2};
use broccoli::prelude::*;

fn main() {
    let mut aabbs = [
        bbox(rec(0isize, 10, 0, 10), 0),
        bbox(rec(15, 20, 15, 20), 1),
        bbox(rec(5, 15, 5, 15), 2),
    ];

    //Create a layer of direction.
    let mut ref_aabbs = aabbs.iter_mut().collect::<Vec<_>>();

    let border = rect(0, 100, 0, 100);

    //Create a DinoTree by picking a starting axis (x or y).
    //This will change the order of the elements in bboxes,
    //but this is okay since we populated it with mutable references.
    let mut tree = broccoli::new(&mut ref_aabbs);

    //Here we query for read-only references so we can pull
    //them out of the closure.
    let mut rect_collisions = Vec::new();
    tree.for_all_intersect_rect(&rect(-5, 1, -5, 1), |a| {
        rect_collisions.push(a);
    });

    assert_eq!(rect_collisions.len(), 1);
    assert_eq!(*rect_collisions[0].get_rec(), rec(0, 10, 0, 10));

    let res = tree.k_nearest_mut(
        vec2(30, 30),
        2,
        &mut (),
        |(), a, b| b.distance_squared_to_point(a).unwrap_or(0),
        |(), a, b| b.rect.as_ref().distance_squared_to_point(a).unwrap_or(0),
        border,
    );
    assert_eq!(res[0].bot, &1);
    assert_eq!(res[1].bot, &2);

    let ray = axgeom::Ray {
        point: vec2(-10, 1),
        dir: vec2(1, 0),
    };
    let res = tree.raycast_mut(
        ray,
        &mut (),
        |(), ray, r| ray.cast_to_rect(r),
        |(), ray, b| ray.cast_to_rect(b.get_rec().as_ref()),
        border,
    );
    assert_eq!(res.unwrap().0[0], &0);
}
