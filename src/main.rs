extern crate nalgebra as na;
extern crate image;
use image::{DynamicImage, ExtendedColorType::Rgb1, GenericImage, GenericImageView, GrayImage, ImageBuffer, ImageFormat, Luma, Rgb, codecs::pnm::ArbitraryTuplType::RGB, imageops};

fn main() {

    let mut img = image::open("src/images/IMG_20230504_090854.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });

    // Dimensoes
    let (width, height) = img.dimensions();

    //Para posterior comparacao 
    println!("Dimensoes = {width} {height}");

    // Grayscale
    let mut cinza = img.to_luma8();

    /* ====================================================================================================================================================================================================================
    Algoritmo de otsu 
    */
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

    let mut cinza_2 = GrayImage::from_pixel(width+2, height+2, Luma([0]));
    
    
    /* ====================================================================================================================================================================================================================
    Procedimento para extrair contornos, recebe uma imagem no formato GrayImage, altura e largura em u32.
    Retorna a imagem resultante do processo de extracao de contornos: "a" - ("a" erodida por "b")
    sendo "b" um elemento estruturante em cruz simulado por comparacao de coordenadas
    */
    fn border_mask(a: &mut GrayImage, b: &mut GrayImage, w: u32, h: u32){

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

        // A imagem a recebe a mascara
        // Insercao da imagem a dentro do fundo de padding
        for y in 1..(h+1) {
            for x in 1..(w+1) {
                b.put_pixel(x,y, image::Luma([eroded_a.get_pixel(x-1,y-1).0[0]]));
            }
        }

    }
    
    border_mask(&mut cinza, &mut cinza_2, width, height); // Cria a mascara sobre a imagem cinza

    /* ==================================================================================================================================================================================================================== 
    Seguidor de bordas, recebe a imagem da máscara (grayscale/GrayImage), a imagem original, altura e largura
    insere bordas na imagem original e retorna as listas de bordas encontradas
    */ 
    fn border_follower(imagem: &mut GrayImage, imagem2: &mut DynamicImage, w :u32, h :u32){

        let mut lista_coords: Vec<Vec<(u32,u32)>> = Vec::with_capacity(8); // Lista de listas

        let mut counter; 
        for mut y in 1..(h+1){ // Percorre a imagem
            for mut x in 1..(w+1){

                if imagem.get_pixel(x,y).0[0]==255{ // Pixel upper-left most

                    let mut border: Vec<(u32, u32)> = Vec::with_capacity(1000); 
                    let coords_ini = (x,y);
                    imagem2.put_pixel(x-1,y-1,image::Rgba::<u8>([255,0,0,255]));
                    border.push(coords_ini); 

                    counter=0; 

                    let offsets_horario: [(u32, u32); 8] = [
                        (x, y-1), 
                        (x+1, y-1),
                        (x+1, y), 
                        (x+1, y+1), 
                        (x, y+1), 
                        (x-1, y+1), 
                        (x-1, y), 
                        (x-1, y-1)
                    ];  

                    for &(dx, dy) in &offsets_horario {
                        if imagem.get_pixel(dx,dy).0[0]==255{
                            counter+=1;
                            imagem.put_pixel(x,y,image::Luma([0]));
                            x=dx;
                            y=dy;
                            imagem2.put_pixel(x,y,image::Rgba::<u8>([255,0,0,255]));
                            border.push((x,y));
                        }
                        else if (dx,dy) == coords_ini && counter>300{ // Borda deve ser cíclia e ter mais de 50 pixels
                            break;
                        }
                    }
                    lista_coords.push(border);
                    x=coords_ini.0; // x e y voltam para o upper-left most
                    y=coords_ini.1;
                }
            }
        }
    }

    border_follower(&mut cinza_2, &mut img, width, height);
    
    img.save_with_format("src/images/Bordeada.bmp", ImageFormat::Bmp).ok();

}
