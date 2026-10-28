extern crate nalgebra as na;
extern crate image;
use image::{GenericImageView, GrayImage, ImageFormat, imageops, Luma};

fn main() {

    let img = image::open("src/images/IMG_20230504_090854.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });

    // Dimensoes
    let (width, height) = img.dimensions();

    //Para posterior comparacao 
    println!("Dimensoes = {width} {height}");

    // Grayscale
    let mut cinza = img.into_luma8();

    // let size = 1;

    // Função para limiarizar a imagem, recebe a referencia da imagem e o total de pixels, retorna o limiar em u64 (Nao usada)
    // fn lim_img(imagem: &mut GrayImage, size:u32) {

    //     let (width, height) = imagem.dimensions();
    //     if width == 0 || height == 0 {
    //         return;
    //     }

    //     let block_height = height/size;
    //     let block_width = width/size;
    //     let block_size = block_width*block_height;
    //     let mut t:u32;
    //     let mut soma: u32;
    //     let mut soma2: u32; 
    //     let mut qtd_menores: u32;
    //     let mut qtd_maiores: u32;
    //     let mut t2: u32; 
    //     let mut t3: u32;
    //     let mut novot: u32;

    //     for i in 0..size{ // Para cada bloco
    //         for j in 0..size{

    //             // Calcula a soma da intensidade do bloco
    //             soma = 0;
    //             for y in i*block_height..((i+1)*block_height){
    //                 for x in j*block_width..((j+1)*block_width){
    //                     soma += imagem.get_pixel(x,y).0[0] as u32;
    //                 }
    //             }

    //             // Limiar inicial 
    //             t = soma/block_size; 
                
    //             // Calculo adaptativo do limiar t
    //             loop{
    //                 soma = 0; 
    //                 soma2 = 0;
    //                 qtd_menores = 0;
    //                 qtd_maiores = 0;
                
    //                 for y in i*block_height..((i+1)*block_height){
    //                     for x in j*block_width..((j+1)*block_width){
    //                         if imagem.get_pixel(x,y).0[0] as u32 <= t {   // Se menor ou igual a t
    //                             soma += imagem.get_pixel(x,y).0[0] as u32;
    //                             qtd_menores += 1;
    //                         } else {                                      // Se maiores que t
    //                             soma2 += imagem.get_pixel(x,y).0[0] as u32;
    //                             qtd_maiores += 1;
    //                         }
    //                     }
    //                 } 
                        
    //                 t2 = if qtd_menores > 0 { soma / qtd_menores } else { 0 };
    //                 t3 = if qtd_maiores > 0 { soma2 / qtd_maiores } else { 0 };

    //                 novot= (t2+t3)/2;

    //                 if t==novot{ 
    //                     break;
    //                 }
    //                 t=novot;
    //             }
    //             println!("Limiar calculado: {t}");

    //             for y in i*block_height..((i+1)*block_height){ // Limiariza a imagem
    //                 for x in j*block_width..((j+1)*block_width){
    //                     if imagem.get_pixel(x,y).0[0] > t as u8 {
    //                         imagem.put_pixel(x,y, Luma([0])); 
    //                     } else {
    //                         imagem.put_pixel(x,y, Luma([255]));
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     return;   
    // }

    // lim_img(&mut cinza, size);

    // ALgoritmo de otsu
    fn otsu(img_cinza: &mut GrayImage) {
        let total_pixels = (img_cinza.width() * img_cinza.height()) as f64;

        // 1. Histograma (frequência de cada nível de cinza)
        let mut histogram = [0u64; 256];
        for pixel in img_cinza.pixels() {
            histogram[pixel[0] as usize] += 1;
        }

        // 2. Probabilidades (pi) e Média Global (Ut)
        let mut pi = [0.0f64; 256];
        let mut ut = 0.0f64;
        for i in 0..256 {
            pi[i] = histogram[i] as f64 / total_pixels;
            ut += i as f64 * pi[i];
        }

        let mut w_k = 0.0f64; // Soma acumulada das probabilidades (w)
        let mut u_k = 0.0f64; // Soma acumulada do produto nível * probabilidade (u)
        let mut max_variance = -1.0f64;
        let mut limiar = 0u8;

        // 3. Busca do limiar k que maximiza a variância entre classes
        for k in 0..256 {
            w_k += pi[k];
            u_k += k as f64 * pi[k];

            let w_background = w_k;
            let w_foreground = 1.0 - w_k;

            // Intervalo efetivo: w(k) * (1 - w(k)) > 0
            if w_background > 0.0 && w_foreground > 0.0 {
                // Equação da variância entre classes: ((Ut * w - u)^2) / (w * (1 - w))
                let numerator = ut * w_background - u_k;
                let variance = (numerator * numerator) / (w_background * w_foreground);

                // Mantém o comportamento do argmax (primeiro máximo encontrado)
                if variance > max_variance {
                    max_variance = variance;
                    limiar = k as u8;
                }
            }
        }

        for pixel in img_cinza.pixels_mut() {
            if pixel[0] > limiar {
                pixel[0] = 0;
            } else {
                pixel[0] = 255;
            }
        }   
    }

        otsu(&mut cinza);
    
    cinza.save_with_format("src/images/Limiarizado.bmp", ImageFormat::Bmp).ok();

    /*
    Procedimento para extrair contornos, recebe uma imagem "a" no formato GrayImage, altura e largura em u32.
    Retorna a imagem resultante do processo de extracao de contornos: "a" - ("a" erodida por "b")
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
                   a_padded.get_pixel(x,y-1).0[0] == 255 && a_padded.get_pixel(x, y+1).0[0] == 255 // &&
                //    a_padded.get_pixel(x-1, y-1).0[0] == 255 && a_padded.get_pixel(x-1, y+1).0[0] == 255 && 
                //    a_padded.get_pixel(x+1,y-1).0[0] == 255 && a_padded.get_pixel(x+1, y+1).0[0] == 255
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

    // Seguidor de bordas
    fn border_follower(imagem: &mut GrayImage, h :u32, w :u32){
        let mut lista_coords: Vec<Vec<(u32,u32)>> = Vec::new();
        for y in 1..h+1{
            for x in 1..w+1{
                if imagem.get_pixel(x,y).0[0]==255{
                    let mut border: Vec<(u32, u32)> = Vec::with_capacity(1000);
                    let mut coords_ini = (x,y);
                    let mut coord_atual = coords_ini;

                    border.push(coords_ini);
                    lista_coords.push(border);  
                    let offsets_horario: [(i32, i32); 8] = [
                        (-1, -1), 
                        ( 0, -1),
                        ( 1, -1), 
                        ( 1,  0), 
                        ( 1,  1), 
                        ( 0,  1), 
                        (-1,  1), 
                        (-1,  0), 
                    ];                
                    let mut b0 = imagem.get_pixel(x,y).0[0];
                    let mut c0 = imagem.get_pixel(x-1,y).0[0];
                    for &(dx, dy) in &offsets_horario {
                        
                    }
                }
            }
        }
    }

}
