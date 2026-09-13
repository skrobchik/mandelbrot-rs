use cutile::prelude::*;

#[cutile::module]
mod kernel {
    use cutile::core::*;

    #[cutile::entry()]
    fn add<const B: i32>(
        z: &mut Tensor<f32, { [B] }>,
        x: &Tensor<f32, { [-1] }>,
        y: &Tensor<f32, { [-1] }>,
    ) {
        let tx = x.load_like(z);
        let ty = y.load_like(z);
        z.store(tx + ty);
    }
}

fn main() -> Result<(), Error> {
    let x = api::ones::<f32>(&[1024]);
    let y = api::ones::<f32>(&[1024]);
    let z = api::zeros::<f32>(&[1024]).partition([128]);

    let (_z, _x, _y) = kernel::add(z, x, y).sync()?;
    Ok(())
}