extern crate image;
use image::{DynamicImage, GrayImage, Luma};

fn grayscale(image: &DynamicImage) -> GrayImage {
    let rgb_image = image.to_rgb8();

    GrayImage::from_fn(rgb_image.width(), rgb_image.height(), |x, y| {
        let [red, green, blue] = rgb_image.get_pixel(x, y).0;
        let gray = 0.299 * f64::from(red) + 0.587 * f64::from(green) + 0.114 * f64::from(blue);

        Luma([gray.round() as u8])
    })
}

fn otsu(img: &mut GrayImage) {
    let total_pixels = (img.width() * img.height()) as f64;

    // Histograma (frequência de cada nível de cinza)
    let mut histogram = [0u64; 256];
    for pixel in img.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    // Segundo Otsu (1979):
    // - probabilidade de cada nível de cinza k: pi = frequencia_k / total_pixels (Equação 1)
    // - média global: Ut = soma dos produtos dos níveis de cinza por suas probabilidades (Equação 8)
    let mut pi = [0.0f64; 256];
    let mut ut = 0.0f64;
    for i in 0..256 {
        pi[i] = histogram[i] as f64 / total_pixels;
        ut += i as f64 * pi[i];
    }

    let mut w_k = 0.0f64;  // w = soma das probabilidades até o nível k (inclusive) (OTSU, 1979, Eq. 2)
    let mut u_k = 0.0f64; // u = soma do produto dos níveis de cinza por suas probabilidades até k (inclusive) (OTSU, 1979, Eq. 7
    let mut max_variance = -1.0f64;
    let mut limiar = 0u8;

    // Busca do limiar k que maximiza a variância entre classes (OTSU, 1979, Eq. 19)
    for k in 0..256 {
        w_k += pi[k];
        u_k += k as f64 * pi[k];
        
        // Otsu (1979) define o que ele chama de "intervalo efetivo"
        // Isto é, valores de k onde => w(k) * (1 - w(k)) > 0 
        let w_background = w_k;
        let w_foreground = 1.0 - w_k;
        if w_background > 0.0 && w_foreground > 0.0 {
            // variância entre classes (OTSU, 1979, Eq. 18)
            let numerator = ut * w_background - u_k;
            let variance = (numerator * numerator) / (w_background * w_foreground);
            if variance > max_variance {
                max_variance = variance;
                limiar = k as u8;
            }
        }
    }

    for pixel in img.pixels_mut() {
        if pixel[0] > limiar {
            pixel[0] = 0;
        } else {
            pixel[0] = 255;
        }
    }
}

fn border_mask(a: &mut GrayImage, b: &mut GrayImage, w: u32, h: u32) {
    let mut a_padded = GrayImage::from_pixel(w + 2, h + 2, Luma([0]));

    // Insercao da imagem a dentro do fundo de padding
    for y in 1..(h + 1) {
        for x in 1..(w + 1) {
            a_padded.put_pixel(x, y, image::Luma([a.get_pixel(x - 1, y - 1).0[0]]));
        }
    }

    let mut eroded_a = GrayImage::from_pixel(w + 2, h + 2, Luma([0]));

    //Erosao com elemento B em cruz
    for y in 1..(h + 1) {
        for x in 1..(w + 1) {
            //Elemento B simulado por comparacao de coordenadas
            if a_padded.get_pixel(x, y).0[0] == 255
                && a_padded.get_pixel(x - 1, y).0[0] == 255
                && a_padded.get_pixel(x + 1, y).0[0] == 255
                && a_padded.get_pixel(x, y - 1).0[0] == 255
                && a_padded.get_pixel(x, y + 1).0[0] == 255
            {
                eroded_a.put_pixel(x, y, image::Luma([255]));
            }
        }
    }

    //Diferença da imagem original
    let iter = eroded_a.as_mut().iter_mut().zip(a_padded.as_raw().iter());

    for (eroded_a, &a_padded) in iter {
        if a_padded == 255 && *eroded_a == 0 {
            *eroded_a = 255;
        } else {
            *eroded_a = 0;
        }
    }

    for y in 1..(h + 1) {
        for x in 1..(w + 1) {
            b.put_pixel(x, y, image::Luma([eroded_a.get_pixel(x - 1, y - 1).0[0]]));
        }
    }
}

fn border_follower(img: &GrayImage) -> Vec<Vec<(u32, u32)>> {
    let (w, h) = img.dimensions();

    let mut visited = vec![false; (w * h) as usize];

    let mut objs_borders: Vec<Vec<(u32, u32)>> = Vec::new();

    let idx = |x: u32, y: u32| (y * w + x) as usize;

    let inside = |x: i32, y: i32| x >= 1 && x <= w as i32 && y >= 1 && y <= h as i32;

    let neighbors: [(i32, i32); 8] = [
        (-1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
    ];

    for y in 1..h {
        for x in 1..w {
            if img.get_pixel(x, y).0[0] != 255 || visited[idx(x, y)] {
                continue;
            }

            let mut queue = vec![(x, y)];
            visited[idx(x, y)] = true;
            let mut upper_left = (x, y);
            let mut pos = 0;

            while pos < queue.len() {
                let (atual_x, atual_y) = queue[pos];
                pos += 1;

                if (atual_y, atual_x) < (upper_left.1, upper_left.0) {
                    upper_left = (atual_x, atual_y);
                }

                for &(dx, dy) in &neighbors {
                    let nx = atual_x as i32 + dx;
                    let ny = atual_y as i32 + dy;

                    if inside(nx, ny) {
                        if img.get_pixel(nx as u32, ny as u32).0[0] == 255
                            && !visited[idx(nx as u32, ny as u32)]
                        {
                            visited[idx(nx as u32, ny as u32)] = true;
                            queue.push((nx as u32, ny as u32));
                        }
                    }
                }
            }

            let mut border = vec![upper_left];
            let mut b = upper_left; // Pixel corrente
            let mut c;

            let first = (0..8).find_map(|i| {
                let nx = b.0 as i32 + neighbors[i].0;
                let ny = b.1 as i32 + neighbors[i].1;
                if inside(nx, ny) && img.get_pixel(nx as u32, ny as u32).0[0] == 255 {
                    let prev = (i + 7) % 8;
                    Some((
                        (nx as u32, ny as u32),
                        (
                            (b.0 as i32 + neighbors[prev].0) as u32,
                            (b.1 as i32 + neighbors[prev].1) as u32,
                        ),
                    ))
                } else {
                    None
                }
            });


            if let Some((first_b, first_c)) = first {
                b = first_b;
                c = first_c;
                border.push(b);

                while b != upper_left {
                    let indice_c = neighbors
                        .iter()
                        .position(|&neighbor| {
                            b.0 as i32 + neighbor.0 == c.0 as i32
                                && b.1 as i32 + neighbor.1 == c.1 as i32
                        })
                        .unwrap_or(0);

                    let next = (0..8).find_map(|offset| {
                        let i = (indice_c + offset) % 8;
                        let nx = b.0 as i32 + neighbors[i].0;
                        let ny = b.1 as i32 + neighbors[i].1;
                        if inside(nx, ny) && img.get_pixel(nx as u32, ny as u32).0[0] == 255 {
                            let prev = (i + 7) % 8;
                            Some((
                                (nx as u32, ny as u32),
                                (
                                    (b.0 as i32 + neighbors[prev].0) as u32,
                                    (b.1 as i32 + neighbors[prev].1) as u32,
                                ),
                            ))
                        } else {
                            None
                        }
                    });

                    let Some((next_b, next_c)) = next else {
                        break;
                    };
                    c = next_c;
                    b = next_b;
                    border.push(b);

                    if border.len() > (w as usize * h as usize) + 1 {
                        break;
                    }
                }
            }

            if border.len() > 500 {
                objs_borders.push(border);
            }
        }
    }

    objs_borders
}

pub fn process_image(image: &DynamicImage) -> Vec<Vec<(u32, u32)>> {
    let mut gray_image = grayscale(image);
    otsu(&mut gray_image);

    let (w, h) = gray_image.dimensions();
    let mut border_mask_image = GrayImage::from_pixel(w + 2, h + 2, Luma([0]));
    border_mask(&mut gray_image, &mut border_mask_image, w, h);
    let borders = border_follower(&border_mask_image);

    borders
}
