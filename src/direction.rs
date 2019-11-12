use nalgebra as na;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up, Right, Down, Left,
    UpLeft, UpRight,
    DownLeft, DownRight,
    Neutral,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Neutral
    }
}

impl Direction {
    // TODO make const fn
    fn x(&self) -> i8 {
        use Direction::*;
        match self {
            Left | UpLeft | DownLeft => 1,
            Right | UpRight | DownRight => -1,
            Neutral | Up | Down => 0,
        }
    }
    // TODO make const fn
    fn y(&self) -> i8 {
        use Direction::*;
        match self {
            Up | UpLeft | UpRight => 1,
            Down | DownLeft | DownRight => -1,
            Neutral | Left | Right => 0,
        }
    }
}

impl Direction {
    pub fn to_na_vec(&self) -> na::Vector2<f64> {
        let mut v = nalgebra::Vector2::new(self.x() as f64, self.y() as f64);
        v.try_normalize_mut(1e-9);
        v
    }
}

impl Direction {
    pub fn shift_up(&self) -> Direction {
        use Direction::*;
        match self {
            Neutral => Up,
            Down => Neutral,
            Right => UpRight,
            Left => UpLeft,
            DownLeft => Left,
            DownRight => Right,
            Up | UpLeft | UpRight => *self,
        }
    }
    pub fn shift_down(&self) -> Direction {
        use Direction::*;
        match self {
            Neutral => Down,
            Up => Neutral,
            Right => DownRight,
            Left => DownLeft,
            UpLeft => Left,
            UpRight => Right,
            Down | DownLeft | DownRight => *self,
        }
    }
    pub fn shift_left(&self) -> Direction {
        use Direction::*;
        match self {
            Neutral => Left,
            Right => Neutral,
            Down => DownLeft,
            Up => UpLeft,
            UpRight => Up,
            DownRight => Down,
            Left | UpLeft | DownLeft => *self,
        }
    }
    pub fn shift_right(&self) -> Direction {
        use Direction::*;
        match self {
            Neutral => Right,
            Left => Neutral,
            Down => DownRight,
            Up => UpRight,
            UpLeft => Up,
            DownLeft => Down,
            Right | UpRight | DownRight => *self,
        }
    }
}

impl From<Direction> for mint::Vector2<f64> {
    fn from(d: Direction) -> Self {
        d.to_na_vec().into()
    }
}

// impl From<Direction> for nalgebra::Vector2<f64> {
//     fn from(d: Direction) -> Self {
//         nalgebra::Vector2::new(d.x() as f64, d.y() as f64)
//     }
// }
// impl Into<nalgebra::Vector2<f32>> for Direction {
//     fn into(self) -> nalgebra::Vector2<f32> {
//         nalgebra::Vector2::new(self.x() as f32, self.y() as f32)
//     }
// }
