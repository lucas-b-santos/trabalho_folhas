use std::print;
extern crate nalgebra as na;
extern crate image;
use image::{GenericImageView, GrayImage, ImageFormat, imageops, Luma};

fn main() {

    let img = image::open("src/images/IMG_20230504_090659.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });

    //Dimensoes
    let (width, height) = img.dimensions();
    let total_pixels = (width * height) as u64;

    println!("Dimensoes = {width} {height}");//Para posterior comparacao 

    let mut cinza = img.into_luma8(); //Grayscale

    //Função para calular limiar, recebe a referencia da imagem e o total de pixels, retorna o limiar em u64

    fn lim_calc(imagem: &GrayImage, total_pixels: u64) -> u64 {

        let mut soma: u64 = imagem.as_raw() // Somatorio dos valores da imagem
        .iter()
        .map(|&valor| valor as u64)
        .sum();

        let mut t: u64 = (soma / total_pixels) as u64; //Limiar

        loop{ //Repetir até t parar de ser alterado

            soma = 0; 

            let mut soma2: u64 = 0;
            let mut qtd_menores: u64 = 0;
            let mut qtd_maiores: u64 = 0;
        
            for &valor in imagem.as_raw() { 
                if valor as u64 <= t {      // Se menor ou igual a t
                    soma += valor as u64;
                    qtd_menores += 1;
                } else {                    // Se maiores que t
                    soma2 += valor as u64;
                    qtd_maiores += 1;
                }
            }

            let t2: u64 = if qtd_menores > 0 { soma / qtd_menores } else { 0 };
            let t3: u64 = if qtd_maiores > 0 { soma2 / qtd_maiores } else { 0 };

            let novot: u64 = (t2+t3)/2;

            if t!=novot{
                t=novot;
                break;
            }
        }

        return t;
    }

    let lim = lim_calc(&cinza, total_pixels); //Calcula limiar

    for valor in cinza.iter_mut() { //Limiarização
        if *valor > lim as u8 {
            *valor = 0; 
        } else {
            *valor = 255;
        }
    }
    cinza.save_with_format("src/images/Limiarizado.bmp", ImageFormat::Bmp).ok();

    /*
    Procedimento para extrair contornos, recebe uma imagem "a" no formato GrayImage, altura e largura em u32.
    Retorna a imagem resuultante do processo de extracao de contornos: "a" - ("a" erodida por "b")
    sendo "b" um elemento estruturante em cruz simulado por comparacao de coordenadas
    */
    fn border_mask(a: &mut GrayImage, w: u32, h: u32){

        let mut a_padded = GrayImage::from_pixel(w+2, h+2, Luma([255]));

        // Insercao da imagem a dentro do fundo de padding
        for y in 1..(h+1) {
            for x in 1..(w+1) {
                a_padded.put_pixel(x,y, image::Luma([a.get_pixel(x-1,y-1).0[0]]));
            }
        }

        //imageops::replace(&mut a_padded, a, 1, 1); //Imagem com zero padding
        
        let mut eroded_a = GrayImage::from_pixel(w, h, Luma([255])); // Nova imagem preta

        // Erosão com elemento B em cruz (Tornou-se uma Dilatacao apesar da logica ser de erosao)
        for y in 1..(h+1) {
            for x in 1..(w+1) {
                if a_padded.get_pixel(x, y).0[0] == 0 && a_padded.get_pixel(x-1, y).0[0] == 0 && a_padded.get_pixel(x+1, y).0[0] == 0 && a_padded.get_pixel(x,y-1).0[0] == 0 && a_padded.get_pixel(x, y+1).0[0] == 0  { //Elemento B simulado pro comparação posicional
                    eroded_a.put_pixel(x-1,y-1, image::Luma([0]));
                }
                else{
                    eroded_a.put_pixel(x-1,y-1, image::Luma([255]));
                }
            }
        }
        
        eroded_a.save_with_format("src/images/Dilatada.bmp", ImageFormat::Bmp).ok();
    
        //Adição com imagem original
        for y in 0..(h) {
            for x in 0..(w) {
                if eroded_a.get_pixel(x, y).0[0] == 255 && a.get_pixel(x, y).0[0] == 0{
                    a.put_pixel(x, y, image::Luma([255]));
                }
                else{
                    a.put_pixel(x, y, image::Luma([0]));
                }
            }
        }
        a.save_with_format("src/images/Mascara.bmp", ImageFormat::Bmp).ok(); 
    }
    
    border_mask(&mut cinza, width, height); // Cria a mascara sobre a imagem cinza

    //Seguidor de bordas
    fn border_follower(imagem: &mut GrayImage, h :u32, w :u32){

    }

}
