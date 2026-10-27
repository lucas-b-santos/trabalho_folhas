extern crate nalgebra as na;
extern crate image;
use image::{GenericImageView, GrayImage, ImageFormat, imageops, Luma};

fn main() {

    let img = image::open("src/images/IMG_20230504_090659.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });

    // Dimensoes
    let (width, height) = img.dimensions();

    //Para posterior comparacao 
    println!("Dimensoes = {width} {height}");

    // Grayscale
    let mut cinza = img.into_luma8(); 

    // Função para limiarizar a imagem, recebe a referencia da imagem e o total de pixels, retorna o limiar em u64
    fn lim_img(imagem: &mut GrayImage, width: u32, height: u32) {
        
        let block_height = height/4;
        let block_width = width/8;
        let block_size = block_width*block_height;
        let mut t:u32;
        let mut soma: u32;
        let mut soma2: u32;
        let mut qtd_menores: u32;
        let mut qtd_maiores: u32;
        let mut t2: u32;
        let mut t3: u32;
        let mut novot: u32;
        for i in 0..3{ // Para cada bloco
            for j in 0..7{

                // Calcula a soma da intensidade do bloco
                soma = 0;
                for y in i*block_height..(i*block_height)+block_height{
                    for x in j*block_width..(j*block_width)+block_width{
                        soma += imagem.get_pixel(x,y).0[0] as u32;
                    }
                }

                // Limiar inicial 
                t = soma/block_size; 
                
                // Calculo adaptativo do limiar t
                loop{
                    soma = 0; 

                    soma2 = 0;
                    qtd_menores = 0;
                    qtd_maiores = 0;
                
                    for y in i*block_height..(i*block_height)+block_height{
                    for x in j*block_width..(j*block_width)+block_width{
                            if imagem.get_pixel(x,y).0[0] as u32 <= t {   // Se menor ou igual a t
                                soma += imagem.get_pixel(x,y).0[0] as u32;
                                qtd_menores += 1;
                            } else {                                      // Se maiores que t
                                soma2 += imagem.get_pixel(x,y).0[0] as u32;
                                qtd_maiores += 1;
                            }
                        }
                    } 
                        
                    t2 = if qtd_menores > 0 { soma / qtd_menores } else { 0 };
                    t3 = if qtd_maiores > 0 { soma2 / qtd_maiores } else { 0 };

                    novot= (t2+t3)/2;

                    if t!=novot{ 
                        t=novot;
                        break;
                    }
                }
                println!("Limiar calculado: {t}");

                for y in i*block_height..(i*block_height)+block_height{ // Limiariza a imagem
                    for x in j*block_width..(j*block_width)+block_width{
                        if imagem.get_pixel(x,y).0[0] > t as u8 {
                            imagem.put_pixel(x,y, Luma([0])); 
                        } else {
                            imagem.put_pixel(x,y, Luma([255]));
                        }
                    }
                }
            }
            println!("i mudado");
        }   
    }

    lim_img(&mut cinza, width, height);
    cinza.save_with_format("src/images/Limiarizado.bmp", ImageFormat::Bmp).ok();

    /*
    Procedimento para extrair contornos, recebe uma imagem "a" no formato GrayImage, altura e largura em u32.
    Retorna a imagem resuultante do processo de extracao de contornos: "a" - ("a" erodida por "b")
    sendo "b" um elemento estruturante em cruz simulado por comparacao de coordenadas
    */
    fn border_mask(a: &mut GrayImage, w: u32, h: u32){

        let mut a_padded = GrayImage::from_pixel(w+2, h+2, Luma([0])); // Nova imagem preta

        // Insercao da imagem a dentro do fundo de padding
        for y in 1..(h+1) {
            for x in 1..(w+1) {
                a_padded.put_pixel(x,y, image::Luma([a.get_pixel(x-1,y-1).0[0]]));
            }
        }

        let mut eroded_a = GrayImage::from_pixel(w+2, h+2, Luma([0])); // Nova imagem preta

        //Erosao com elemento B em cruz
        for y in 1..(h+1) {
            for x in 1..(w+1) {
                //Elemento B simulado por comparacao de coordenadas
                if a_padded.get_pixel(x, y).0[0] == 255 &&  
                   a_padded.get_pixel(x-1, y).0[0] == 255 && a_padded.get_pixel(x+1, y).0[0] == 255 && 
                   a_padded.get_pixel(x,y-1).0[0] == 255 && a_padded.get_pixel(x, y+1).0[0] == 255 
                   { 
                    eroded_a.put_pixel(x,y, image::Luma([255]));
                }        
            }
        }
    
        eroded_a.save_with_format("src/images/Erodida.bmp", ImageFormat::Bmp).ok();
        
        //Diferença da imagem original
        let iter = eroded_a.as_mut().iter_mut().zip(a_padded.as_raw().iter());

        for (eroded_a, &a_padded) in iter {
            if a_padded == 255 && *eroded_a == 0{
                *eroded_a= 255;
            } 
            else{
                *eroded_a=0;
            }
        }

        eroded_a.save_with_format("src/images/Mascara.bmp", ImageFormat::Bmp).ok();
        
    }
    
    border_mask(&mut cinza, width, height); // Cria a mascara sobre a imagem cinza

    //Seguidor de bordas
    // fn border_follower(imagem: &mut GrayImage, h :u32, w :u32){
    //     let mut marcado = (0,0);
    //     let vector![];

    //     for y in 1..h+1{
    //         for x in 1..w+1{
    //             if imagem.get_pixel(x,y).0[0]==255{

    //             }
    //         }
    //     }
    // }

}
