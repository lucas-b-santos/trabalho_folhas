use std::print;
extern crate nalgebra as na;
extern crate image;
use image::{GenericImageView, GrayImage, ImageFormat};

fn main() {

    let img = image::open("src/IMG_20230504_090659.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });

    let (width,height) = img.dimensions();
    let _total_pixels = (width * height) as u64;
    let mut _cinza = img.into_luma8();

    //Calular limiar
    fn lim_calc(imagem: &GrayImage, total_pixels: u64) -> u64 {

        let mut soma: u64 = imagem.as_raw()
        .iter()
        .map(|&valor| valor as u64)
        .sum();

        let mut _t: u64 = (soma / total_pixels) as u64;

        loop{ //Repetir até _t parar de ser alterado

            soma = 0; 

            let mut soma2: u64 = 0;
            let mut qtd_menores: u64 = 0;
            let mut qtd_maiores: u64 = 0;
        
            for &valor in imagem.as_raw() { 
                if valor as u64 <= _t { // Se menor ou igual a _t
                    soma += valor as u64;
                    qtd_menores += 1;
                } else {        // Se maiores que _t
                    soma2 += valor as u64;
                    qtd_maiores += 1;
                }
            }

            let _t2: u64 = if qtd_menores > 0 { soma / qtd_menores } else { 0 };
            let _t3: u64 = if qtd_maiores > 0 { soma2 / qtd_maiores } else { 0 };

            let _novot: u64 = (_t2+_t3)/2;

            if _t!=_novot{
                _t=_novot;
                break;
            }
        }

        return _t;
    }

    let mut _lim = lim_calc(&_cinza, _total_pixels); //Calcula limiar

    print!("Limiar calculado = {_lim}");

    for valor in _cinza.iter_mut() { //Limiarização
        if *valor > _lim as u8 {
            *valor = 255;
        } else {
            *valor = 0;
        }
    }
    _cinza.save_with_format("src/Limiarizado.bmp", ImageFormat::Bmp).ok();

}
