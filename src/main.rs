extern crate nalgebra as na;
extern crate image;
use image::{DynamicImage, GenericImage, GenericImageView, GrayImage, ImageFormat, Luma};

fn main() {

    let mut img = image::open("src/images/IMG_20230504_090659.bmp").unwrap_or_else(|e| {
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

        let mut a_padded = GrayImage::from_pixel(w+2, h+2, Luma([0])); 

        // Insercao da imagem a dentro do fundo de padding
        for y in 1..(h+1) {
            for x in 1..(w+1) {
                a_padded.put_pixel(x,y, image::Luma([a.get_pixel(x-1,y-1).0[0]]));
            }
        }

        let mut eroded_a = GrayImage::from_pixel(w+2, h+2, Luma([0])); 

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

        for y in 1..(h+1) {
            for x in 1..(w+1) {
                b.put_pixel(x,y, image::Luma([eroded_a.get_pixel(x-1,y-1).0[0]]));
            }
        }

    }
    
    border_mask(&mut cinza, &mut cinza_2, width, height); // Cria a mascara sobre a imagem cinza

    let mut lista_coords: Vec<Vec<(u32, u32)>> = Vec::new(); // Lista de objetos

    /* ==================================================================================================================================================================================================================== 
    Seguidor de bordas, recebe a imagem da máscara (grayscale/GrayImage), a imagem original, altura e largura
    insere bordas na imagem original e retorna as listas de bordas encontradas
    */ 
     fn border_follower(imagem: &mut GrayImage, imagem2: &mut DynamicImage, w: u32, h: u32, lista_coords: &mut Vec<Vec<(u32, u32)>> ) {

        let width = w + 2;
        let height = h + 2;
        let mut visitado = vec![false; (width * height) as usize];
        
        let indice = |x: u32, y: u32| (y * width + x) as usize; 
        
        let dentro = |x: i32, y: i32| {
            x >= 1 && x <= w as i32 && y >= 1 && y <= h as i32
        };

        let vizinhos: [(i32, i32); 8] = [
            (-1, 0),  
            (-1, -1), 
            (0, -1),  
            (1, -1),  
            (1, 0),   
            (1, 1),   
            (0, 1),   
            (-1, 1), 
        ];
        
        for y in 1..h+1 { 

            for x in 1..w+1 {
                
                if imagem.get_pixel(x, y).0[0] != 255 || visitado[indice(x, y)] { 
                    continue;
                }

                let mut fila = vec![(x, y)];
                visitado[indice(x, y)] = true; 
                let mut upper_left = (x, y); 
                let mut pos = 0; 

                while pos < fila.len() { 

                    let (atual_x, atual_y) = fila[pos];
                    pos += 1;

                    if (atual_y, atual_x) < (upper_left.1, upper_left.0) {  
                        upper_left = (atual_x, atual_y); 
                    }

                    for &(dx, dy) in & vizinhos { 
                        let nx = atual_x as i32 + dx; 
                        let ny = atual_y as i32 + dy;

                        if dentro(nx, ny) { 

                            if imagem.get_pixel(nx as u32, ny as u32).0[0] == 255 
                                && !visitado[indice(nx as u32, ny as u32)] 
                            {
                                visitado[indice(nx as u32, ny as u32)] = true; 
                                fila.push((nx as u32, ny as u32));
                            }
                        }
                    }
                }

                let mut borda = vec![upper_left]; 
                let mut b = upper_left; // Pixel corrente
                let mut c;

                let primeiro = (0..8).find_map(|passo| {
                    let nx = b.0 as i32 + vizinhos[passo as usize].0;
                    let ny = b.1 as i32 + vizinhos[passo as usize].1;
                    if dentro(nx, ny) && imagem.get_pixel(nx as u32, ny as u32).0[0] == 255 {
                        let anterior = (passo + 7) % 8;
                        Some((
                            (nx as u32, ny as u32),
                            (
                                (b.0 as i32 + vizinhos[anterior].0) as u32,
                                (b.1 as i32 + vizinhos[anterior].1) as u32,
                            ),
                        ))
                    } else {
                        None
                    }
                });

                if let Some((primeiro_b, primeiro_c)) = primeiro {
                    b = primeiro_b;
                    c = primeiro_c;
                    borda.push(b);

                    while b != upper_left {
                        let indice_c = vizinhos
                            .iter()
                            .position(|&vizinho| {
                                b.0 as i32 + vizinho.0 == c.0 as i32
                                    && b.1 as i32 + vizinho.1 == c.1 as i32
                            })
                            .unwrap_or(0);

                        let proximo = (0..8).find_map(|offset| {
                            let passo = (indice_c + offset) % 8;
                            let nx = b.0 as i32 + vizinhos[passo].0;
                            let ny = b.1 as i32 + vizinhos[passo].1;
                            if dentro(nx, ny)
                                && imagem.get_pixel(nx as u32, ny as u32).0[0] == 255
                            {
                                let anterior = (passo + 7) % 8;
                                Some((
                                    (nx as u32, ny as u32),
                                    (
                                        (b.0 as i32 + vizinhos[anterior].0) as u32,
                                        (b.1 as i32 + vizinhos[anterior].1) as u32,
                                    ),
                                ))
                            } else {
                                None
                            }
                        });

                        let Some((proximo_b, proximo_c)) = proximo else {
                            break;
                        };
                        c = proximo_c;
                        b = proximo_b;
                        borda.push(b);

                        if borda.len() > (w as usize * h as usize) + 1 {
                            break;
                        }
                    }
                }

                if borda.len()>300{
                    for &(px, py) in &borda {
                    imagem2.put_pixel(px - 1, py - 1, image::Rgba([255, 0, 0, 255]));
                    }
                    lista_coords.push(borda);
                }   
            }
        }
    }

    border_follower(&mut cinza_2, &mut img, width, height, &mut lista_coords);
    
    img.save_with_format("src/images/Bordeada.bmp", ImageFormat::Bmp).ok();

    for i in 0..lista_coords.len(){
        let numero = lista_coords[i].len();
        let index = i+1;
        println!("Objeto {index}: {numero}");
    }

}
