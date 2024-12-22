pub mod shapes;

use clap::{Args, Parser, Subcommand};
use shapes::{Shape, ThreeDShape, TwoDShape};

#[derive(Parser, Debug)]
#[clap(name = "shape_calculator", about = "Calculate the area, volume or perimeter of a shape")]
struct Cli {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[clap(name = "area")]
    Area(AreaArgs),
    #[clap(name = "volume")]
    Volume(VolumeArgs),
    #[clap(name = "perimeter")]
    Perimeter(PerimeterArgs),
}

#[derive(Debug, Args)]
struct AreaArgs {
    #[command(subcommand)]
    shape: ShapeAreaArgs,
}

#[derive(Debug, Args)]
struct VolumeArgs {
    #[command(subcommand)]
    shape: ShapeVolumeArgs,
}

#[derive(Debug, Args)]
struct PerimeterArgs {
    #[command(subcommand)]
    shape: ShapePerimeterArgs,
}

#[derive(Debug, Clone, Subcommand)]
enum ShapeAreaArgs {
    Square { side: f64 },
    Circle { radius: f64 },
    Triangle { base: f64, height: f64 },
    Rectangle { height: f64, width: f64 },
    Sphere { radius: f64 },
    Cilinder { radius: f64, height: f64 },
    Cone { radius: f64, height: f64 },
    Cube { side: f64 },
    Tetrahedron { side: f64 },
}

#[derive(Debug, Clone, Subcommand)]
enum ShapeVolumeArgs {
    Sphere { radius: f64 },
    Cilinder { radius: f64, height: f64 },
    Cone { radius: f64, height: f64 },
    Cube { side: f64 },
    Tetrahedron { side: f64 },
}

#[derive(Debug, Clone, Subcommand)]
enum ShapePerimeterArgs {
    Square { side: f64 },
    Circle { radius: f64 },
    Triangle { side1: f64, side2: f64, side3: f64 },
    Rectangle { height: f64, width: f64 },
}

fn main() {
    use Command::*;
    let args: Cli = Cli::parse();

    match args.cmd {
        Area(args) => {
            use ShapeAreaArgs::*;
            let shape = match args.shape {
                Square { side } => Shape::TwoD(TwoDShape::Square { side }),
                Circle { radius } => Shape::TwoD(TwoDShape::Circle { radius }),
                Triangle { base, height } => Shape::TwoD(TwoDShape::Triangle { base, height, side2: 0.0, side3: 0.0 }),
                Rectangle { height, width } => Shape::TwoD(TwoDShape::Rectangle { height, width }),
                Sphere { radius } => Shape::ThreeD(ThreeDShape::Sphere { radius }),
                Cilinder { radius, height } => Shape::ThreeD(ThreeDShape::Cilinder { radius, height }),
                Cone { radius, height } => Shape::ThreeD(ThreeDShape::Cone { radius, height }),
                Cube { side } => Shape::ThreeD(ThreeDShape::Cube { side }),
                Tetrahedron { side } => Shape::ThreeD(ThreeDShape::Tetrahedron { side }),
            };
            let area = match shape.area() {
                Ok(area) => area,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            };
            println!("Area: {}", area);
        }

        Volume(args) => {
            use ShapeVolumeArgs::*;
            let shape = match args.shape {
                Sphere { radius } => Shape::ThreeD(ThreeDShape::Sphere { radius }),
                Cilinder { radius, height } => Shape::ThreeD(ThreeDShape::Cilinder { radius, height }),
                Cone { radius, height } => Shape::ThreeD(ThreeDShape::Cone { radius, height }),
                Cube { side } => Shape::ThreeD(ThreeDShape::Cube { side }),
                Tetrahedron { side } => Shape::ThreeD(ThreeDShape::Tetrahedron { side }),
            };
            let volume = match shape.volume() {
                Ok(volume) => volume,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            };
            println!("Volume: {}", volume);
        }
        Perimeter(args) => {
            use ShapePerimeterArgs::*;
            let shape = match args.shape {
                Square { side } => Shape::TwoD(TwoDShape::Square { side }),
                Circle { radius } => Shape::TwoD(TwoDShape::Circle { radius }),
                Triangle { side1, side2, side3 } => Shape::TwoD(TwoDShape::Triangle { base: side1, height: 0.0, side2, side3 }),
                Rectangle { height, width } => Shape::TwoD(TwoDShape::Rectangle { height, width }),
            };
            let perimeter = match shape.perimeter() {
                Ok(perimeter) => perimeter,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            };
            println!("Perimeter: {}", perimeter);
        }
    }
}
