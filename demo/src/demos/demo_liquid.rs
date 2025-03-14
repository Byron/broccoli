use crate::support::prelude::*;
use duckduckgeo;

use axgeom::Rect;

#[derive(Copy, Clone, Debug)]
pub struct Liquid {
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub acc: Vec2<f32>,
}

impl Liquid {
    pub fn new(pos: Vec2<f32>) -> Liquid {
        let z = vec2same(0.0);

        Liquid {
            pos,
            acc: z,
            vel: z,
        }
    }

    pub fn solve(&mut self, b: &mut Self, radius: f32) -> f32 {
        let diff = b.pos - self.pos;

        let dis_sqr = diff.magnitude2();

        if dis_sqr < 0.00001 {
            self.acc += vec2(1.0, 0.0);
            b.acc -= vec2(1.0, 0.0);
            return 0.0;
        }

        if dis_sqr >= (2. * radius) * (2. * radius) {
            return 0.0;
        }

        let dis = dis_sqr.sqrt();

        //d is zero if barely touching, 1 is overlapping.
        //d grows linearly with position of bots
        let d = 1.0 - (dis / (radius * 2.));

        let spring_force_mag = -(d - 0.5) * 0.02;

        let velociy_diff = b.vel - self.vel;
        let damping_ratio = 0.00027;
        let spring_dampen = velociy_diff.dot(diff) * (1. / dis) * damping_ratio;

        let spring_force = diff * (1. / dis) * (spring_force_mag + spring_dampen);

        self.acc += spring_force;
        b.acc -= spring_force;

        spring_force_mag
    }
}
pub fn make_demo(dim: Rect<f32>) -> Demo {
    let radius = 50.0;

    let mut bots = support::make_rand(2000, dim, |a| Liquid::new(a));

    Demo::new(move |cursor, canvas, _check_naive| {
        
        let mut k = support::distribute(&mut bots, |bot| {
            let p = bot.pos;
            let r = radius;
            Rect::new(p.x - r, p.x + r, p.y - r, p.y + r)
        });

        let mut tree = broccoli::new_par(&mut k);

        tree.find_colliding_pairs_mut_par(move |a, b| {
            let (a, b) = (a.unpack_inner(), b.unpack_inner());
            let _ = a.solve(b, radius);
        });

        let vv = vec2same(100.0);

        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), move |b| {
            let b = b.unpack_inner();
            let _ = duckduckgeo::repel_one(b.pos, &mut b.acc, cursor, 0.001, 100.0);
        });

        tree.for_all_not_in_rect_mut(&dim, move |a| {
            let a = a.unpack_inner();
            duckduckgeo::collide_with_border(&mut a.pos, &mut a.vel, &dim, 0.5);
        });

        for b in bots.iter_mut() {
            b.pos += b.vel;
            b.vel += b.acc;
            b.acc = vec2same(0.0);
        }

        let mut circle = canvas.circles();
        for bot in bots.iter() {
            circle.add(bot.pos.into());
        }
        circle
            .send_and_uniforms(canvas, 2.0)
            .with_color([1.0, 0.6, 0.7, 0.5])
            .draw();
    })
}
