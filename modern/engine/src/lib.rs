// CYS-004: extracted from archived gh-pages commit ace6576; core format unchanged. Review only.
use wasm_bindgen::prelude::*;
use image::{DynamicImage, RgbaImage, Rgba, load_from_memory, ImageError};
use regex::Regex;
use base64::{engine::general_purpose, Engine as _};
fn build_header(author_note: &str, zip_filename: &str, base64_zip: &[u8]) -> Vec<u8> {
    // 1. UTC timestamp (format: YYYYMMDDHHMMSS, 14 digits)
    let iso_string = js_sys::Date::new_0().to_iso_string().as_string().unwrap();
    let cleaned = iso_string
        .replace("T", "")
        .replace(":", "")
        .replace("-", "")
        .replace("Z", "")
        .replace(".", "");
    let compact_timestamp = &cleaned[0..14]; // "20250531192745"
    let mut timestamp_bytes = compact_timestamp.as_bytes().to_vec();
    timestamp_bytes.resize(100, b' '); // right-pad with spaces

    // 2. Author note (100 bytes, right-padded)
    let cleaned_note = if author_note.trim().is_empty() {
        "No author notes were provided."
    } else {
        author_note.trim()
    };
    let mut author_note_bytes = cleaned_note.as_bytes().to_vec();
    author_note_bytes.resize(100, b' ');

    // 3. ZIP filename (100 bytes, right-padded)
    let mut filename_bytes = zip_filename.as_bytes().to_vec();
    filename_bytes.resize(100, b' ');

    // 4. Payload size (base64 length) as 100-byte, left-padded string
    let payload_str = base64_zip.len().to_string();
    let mut payload_bytes = payload_str.as_bytes().to_vec();
    while payload_bytes.len() < 100 {
        payload_bytes.insert(0, b'0');
    }

    // Final 400-byte header
    [
        timestamp_bytes,
        author_note_bytes,
        filename_bytes,
        payload_bytes,
    ]
    .concat()
}







fn build_composite_image(original: &DynamicImage) -> RgbaImage {
    let width: u32 = original.width();
    let height: u32 = original.height();

    // Convert to RGBA
    let original_rgba = original.to_rgba8();

    // Create a new image that's twice as wide
    let mut composite: RgbaImage = RgbaImage::new(width * 2, height);

    // Left half: original color image
    for y in 0..height {
        for x in 0..width {
            let pixel: Rgba<u8> = *original_rgba.get_pixel(x, y);
            composite.put_pixel(x, y, pixel);
        }
    }

    // Right half: grayscale version
    for y in 0..height {
        for x in 0..width {
            let pixel = original_rgba.get_pixel(x, y);
            let gray: u8 = (0.3 * pixel[0] as f32
                          + 0.59 * pixel[1] as f32
                          + 0.11 * pixel[2] as f32) as u8;
            composite.put_pixel(x + width, y, Rgba([gray, gray, gray, pixel[3]]));
        }
    }

    composite
}





fn compute_anchor_seed_and_shuffle(img: &RgbaImage) -> (usize, Vec<usize>, String) {
    let (width, height) = img.dimensions();
    let total_pixels = (width as usize) * (height as usize);

    // 1. Compute anchor index
    let aspect_ratio = if width >= height {
        width as f64 / height as f64
    } else {
        height as f64 / width as f64
    };
    let anchor_index = (total_pixels as f64 / aspect_ratio).floor() as usize;

    // 2. Extract anchor pixel RGBA
    let anchor_x = anchor_index % (width as usize);
    let anchor_y = anchor_index / (width as usize);
    let anchor_pixel = img.get_pixel(anchor_x as u32, anchor_y as u32);

    // 3. Build seed string (e.g., "3072x4096_R255G123B243A255")
    let seed_string = format!(
        "{}x{}_R{}G{}B{}A{}",
        width,
        height,
        anchor_pixel[0],
        anchor_pixel[1],
        anchor_pixel[2],
        anchor_pixel[3]
    );

    // 4. Hash seed string into a u64 seed
    let seed = seed_string
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

    // 5. Generate shuffled pixel indices (excluding anchor)
    let mut indices: Vec<usize> = (0..total_pixels).collect();
    indices.retain(|&i| i != anchor_index);

    use rand::{rngs::StdRng, SeedableRng};
    use rand::seq::SliceRandom;

    let mut rng = StdRng::seed_from_u64(seed);
    indices.shuffle(&mut rng);

    (anchor_index, indices, seed_string)
}


fn embed_bits_lsb(
    img: &mut RgbaImage,
    header: &[u8],
    payload: &[u8],
    shuffled_indices: &[usize],
) -> Result<(), String> {
    let mut all_data = Vec::with_capacity(header.len() + payload.len());
    all_data.extend_from_slice(header);
    all_data.extend_from_slice(payload);

    let total_bits = all_data.len() * 8;
    let required_pixels = (total_bits + 3) / 4;

    if required_pixels > shuffled_indices.len() {
        return Err("Not enough shuffled pixels to embed data.".to_string());
    }

    let width = img.width() as usize;

    for (bit_index, bit_value) in all_data
        .iter()
        .enumerate()
        .flat_map(|(i, byte)| (0..8).map(move |bit| (i * 8 + bit, (byte >> bit) & 1)))
    {
        let pixel_i = bit_index / 4;
        let channel_i = bit_index % 4;
        let img_i = shuffled_indices[pixel_i];

        let x = img_i % width;
        let y = img_i / width;

        let pixel = img.get_pixel_mut(x as u32, y as u32);
        let channel = &mut pixel.0[channel_i];
        *channel = (*channel & !1) | bit_value;
    }

    Ok(())
}




fn split_and_save_fragments(composite: &RgbaImage) -> (RgbaImage, RgbaImage) {
    let (width, height) = composite.dimensions();
    let half_width = width / 2;

    let mut img1 = RgbaImage::new(half_width, height);
    let mut img2 = RgbaImage::new(half_width, height);

    for y in 0..height {
        for x in 0..half_width {
            // Copy left half exactly:
            let px_left = composite.get_pixel(x, y);
            img1.put_pixel(x, y, *px_left);

            // Copy right half exactly (NO re-grayscale):
            let px_right = composite.get_pixel(x + half_width, y);
            img2.put_pixel(x, y, *px_right);
        }
    }

    (img1, img2)
}



fn cloak_zip_to_images(
    base_image: &DynamicImage,
    zip_filename: &str,
    zip_bytes: &[u8],
    author_note: &str,
) -> Result<(RgbaImage, RgbaImage), ImageError> {
    // Step 1: Build the 400-byte metadata header
    let header = build_header(author_note, zip_filename, zip_bytes);

    // download_bitstream_debug("Header_Cloak", &header[0..14]);


    // download_header_debug("Red5Sorcery_Header_Cloak.txt", &header);

    // Step 2: Build the composite image (color + grayscale)
    let mut composite = build_composite_image(base_image);
    // save_debug_image(&composite, "debug_composite_cloak.png");


    // Step 3: Generate pixel shuffle indices using anchor
    let (_anchor_index, shuffled_indices, seed_string) =
        compute_anchor_seed_and_shuffle(&composite);

    // debug
    // download_shuffle_debug("Red5Sorcery_Shuffle_Cloak.txt", shuffled_indices.len(), "CLOAK");

    // download_seed_log(&seed_string); // debug

    // Step 4: Embed header + payload into composite image
    embed_bits_lsb(&mut composite, &header, zip_bytes, &shuffled_indices)
    .map_err(|e| ImageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;


    // save_debug_image(&composite, "debug_composite_cloak.png");



    // Step 5: Split the composite into v1 (color) and v2 (grayscale)
    let (img1, img2) = split_and_save_fragments(&composite);

    Ok((img1, img2))
}


pub fn reconstruct_composite_image(img1: &RgbaImage, img2: &RgbaImage) -> RgbaImage {
    let (width, height) = img1.dimensions();
    assert_eq!(img2.dimensions(), (width, height), "Image halves must match dimensions");

    let mut composite = RgbaImage::new(width * 2, height);
    for y in 0..height {
        for x in 0..width {
            // Left = exactly img1
            composite.put_pixel(x, y, *img1.get_pixel(x, y));
            // Right = exactly img2
            composite.put_pixel(x + width, y, *img2.get_pixel(x, y));
        }
    }
    composite
}



pub fn extract_header_from_lsb(img: &RgbaImage, shuffled_indices: &[usize]) -> [u8; 400] {
    let mut header_bytes = [0u8; 400];
    let mut current_byte: u8 = 0;
    let mut bit_in_byte = 0;

    let (width, _height) = img.dimensions();

    for bit_index in 0..3200 {
        let pixel_index = bit_index / 4;
        let channel_index = bit_index % 4;

        let shuffled_pixel_index = shuffled_indices[pixel_index];
        let x = (shuffled_pixel_index % (width as usize)) as u32;
        let y = (shuffled_pixel_index / (width as usize)) as u32;

        let pixel = img.get_pixel(x, y);
        let lsb = pixel.0[channel_index] & 1;

        // ✅ match LSB-first embed order
        current_byte |= lsb << bit_in_byte;
        bit_in_byte += 1;

        if bit_in_byte == 8 {
            let byte_index = bit_index / 8;
            header_bytes[byte_index] = current_byte;
            current_byte = 0;
            bit_in_byte = 0;
        }
    }

    header_bytes
}




pub struct HeaderFields {
    pub timestamp: String,
    pub author_note: String,
    pub filename: String,
    pub payload_size: usize,
}

pub fn parse_and_validate_header(header: &[u8]) -> Result<HeaderFields, String> {
    if header.len() != 400 {
        return Err("Header must be exactly 400 bytes.".to_string());
    }

    // 1. Timestamp field: first 100 bytes
    let timestamp_raw = &header[0..100];
    let timestamp_str_full = std::str::from_utf8(timestamp_raw)
        .map_err(|_| "Invalid UTF-8 in timestamp field")?;

    // Validate structure: first 14 chars are digits, rest are spaces
    let re = Regex::new(r"^\d{14} {86}$").unwrap();
    if !re.is_match(timestamp_str_full) {
        return Err("Invalid timestamp format: expected 14 digits followed by 86 spaces.".to_string());
    }

    let timestamp = &timestamp_str_full[0..14];

    // 2. Author note: bytes 100–199
    let author_note = std::str::from_utf8(&header[100..200])
        .map_err(|_| "Invalid UTF-8 in author note")?
        .trim()
        .to_string();

    // 3. Filename: bytes 200–299
    let filename = std::str::from_utf8(&header[200..300])
        .map_err(|_| "Invalid UTF-8 in filename")?
        .trim()
        .to_string();

    if filename.is_empty() {
        return Err("Filename cannot be empty.".to_string());
    }

    // 4. Payload size: bytes 300–399 (left-padded with zeroes)
    let payload_size_str = std::str::from_utf8(&header[300..400])
        .map_err(|_| "Invalid UTF-8 in payload size")?
        .trim_start_matches('0'); // Strip leading zeroes

    if payload_size_str.is_empty() {
        return Err("Payload size is invalid or zero.".to_string());
    }

    let payload_size = payload_size_str
        .parse::<usize>()
        .map_err(|_| "Payload size is not a valid number.")?;

    if payload_size == 0 {
        return Err("Payload size must be greater than zero.".to_string());
    }

    Ok(HeaderFields {
        timestamp: timestamp.to_string(),
        author_note,
        filename,
        payload_size,
    })
}


pub fn extract_payload(
    img: &RgbaImage,
    shuffled_indices: &[usize],
    payload_size: usize,
) -> Result<Vec<u8>, String> {
    let total_bits = payload_size * 8;
    let start_bit = 3200; // Skip header bits

    let (width, _height) = img.dimensions();
    let mut bit_buffer = Vec::with_capacity(total_bits);

    for bit_index in 0..total_bits {
        let global_bit_index = start_bit + bit_index;

        let pixel_index = global_bit_index / 4;
        let channel_index = global_bit_index % 4;

        let shuffled_pixel_index = shuffled_indices
            .get(pixel_index)
            .ok_or("Shuffled index out of bounds")?;

        let x = (*shuffled_pixel_index % (width as usize)) as u32;
        let y = (*shuffled_pixel_index / (width as usize)) as u32;

        let pixel = img.get_pixel(x, y);
        let lsb = pixel.0[channel_index] & 1;

        bit_buffer.push(lsb);
    }

    // Convert bits → bytes
    let mut bytes = Vec::with_capacity(payload_size);
    let mut current_byte = 0u8;

    for (i, &bit) in bit_buffer.iter().enumerate() {
        current_byte |= bit << (i % 8);
        if i % 8 == 7 {
            bytes.push(current_byte);
            current_byte = 0;
        }
    }

    Ok(bytes)
}


fn extract_bits_lsb(
    image: &RgbaImage,
    total_bytes: usize,
    shuffled_indices: &[usize],
) -> Vec<u8> {
    let mut result = vec![0u8; total_bytes];
    let total_bits = total_bytes * 8;

    for bit_index in 0..total_bits {
        let pixel_i = bit_index / 4;
        let channel_i = bit_index % 4;
        let img_i = shuffled_indices[pixel_i];

        let x = img_i % image.width() as usize;
        let y = img_i / image.width() as usize;

        let pixel = image.get_pixel(x as u32, y as u32);
        let bit = (pixel.0[channel_i] & 1) != 0;

        if bit {
            result[bit_index / 8] |= 1 << (bit_index % 8);
        }
    }

    result
}





fn error(message: &str) -> JsValue { JsValue::from_str(message) }
fn capacity(w: u32, h: u32) -> Result<usize, JsValue> {
    // Preserve Classic anchor calculation; reject square composites where anchor is out of bounds.
    let cw = w.checked_mul(2).ok_or_else(|| error("Image dimensions overflow."))?;
    if w == 0 || h == 0 || cw == h { return Err(error("Unsupported image dimensions.")); }
    let pixels = (cw as usize).checked_mul(h as usize).ok_or_else(|| error("Image dimensions overflow."))?;
    if pixels < 801 || pixels > 120_000_000 { return Err(error("Image exceeds review-build pixel limits or is too small.")); }
    Ok((pixels - 1) / 2)
}
fn png(img: RgbaImage) -> Result<js_sys::Uint8Array, JsValue> {
    let mut bytes = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
        .map_err(|_| error("PNG encoding failed."))?;
    Ok(js_sys::Uint8Array::from(bytes.as_slice()))
}
#[wasm_bindgen]
pub fn cloak_local(zip: &[u8], filename: &str, cover: &[u8], note: &str) -> Result<js_sys::Array, JsValue> {
    if zip.is_empty() || zip.len() > 40_000_000 { return Err(error("ZIP size outside supported review range.")); }
    if filename.is_empty() || filename.len() > 100 || note.trim().len() > 100 { return Err(error("Filename or author note exceeds 100 UTF-8 bytes.")); }
    if !filename.to_ascii_lowercase().ends_with(".zip") || filename.contains('/') || filename.contains('\\') { return Err(error("Invalid ZIP filename.")); }
    if !zip.starts_with(b"PK\x03\x04") && !zip.starts_with(b"PK\x05\x06") { return Err(error("ZIP signature not recognized.")); }
    match image::guess_format(cover) { Ok(image::ImageFormat::Png | image::ImageFormat::Jpeg) => {}, _ => return Err(error("Cover must be PNG or JPEG.")) }
    let img = load_from_memory(cover).map_err(|_| error("Cover image could not be decoded."))?;
    let cap = capacity(img.width(), img.height())?;
    let encoded = general_purpose::STANDARD.encode(zip);
    if encoded.len() + 400 > cap { return Err(error("Cover image has insufficient capacity for this archive.")); }
    let (colour, gray) = cloak_zip_to_images(&img, filename, encoded.as_bytes(), note).map_err(|_| error("Cloaking failed."))?;
    let result = js_sys::Array::new();
    result.push(&png(colour)?); result.push(&png(gray)?);
    Ok(result)
}
#[wasm_bindgen]
pub fn decloak_local(colour: &[u8], gray: &[u8]) -> Result<js_sys::Array, JsValue> {
    if !colour.starts_with(b"\x89PNG\r\n\x1a\n") || !gray.starts_with(b"\x89PNG\r\n\x1a\n") { return Err(error("Both images must be PNG.")); }
    let a = load_from_memory(colour).map_err(|_| error("Colour PNG could not be decoded."))?.to_rgba8();
    let b = load_from_memory(gray).map_err(|_| error("Grayscale PNG could not be decoded."))?.to_rgba8();
    if a.dimensions() != b.dimensions() { return Err(error("Image dimensions differ.")); }
    let cap = capacity(a.width(), a.height())?;
    let composite = reconstruct_composite_image(&a, &b);
    let (_, indices, _) = compute_anchor_seed_and_shuffle(&composite);
    let header = extract_bits_lsb(&composite, 400, &indices);
    let fields = parse_and_validate_header(&header).map_err(|_| error("Invalid metadata; check the original image pair and order."))?;
    let total = 400usize.checked_add(fields.payload_size).ok_or_else(|| error("Invalid payload length."))?;
    if total > cap || fields.payload_size > 53_333_336 { return Err(error("Invalid or unsupported payload size.")); }
    let stream = extract_bits_lsb(&composite, total, &indices);
    let zip = general_purpose::STANDARD.decode(&stream[400..]).map_err(|_| error("Recovery failed: invalid payload."))?;
    if !zip.starts_with(b"PK\x03\x04") && !zip.starts_with(b"PK\x05\x06") { return Err(error("Recovered ZIP signature not recognized.")); }
    let result = js_sys::Array::new();
    result.push(&js_sys::Uint8Array::from(zip.as_slice()));
    result.push(&JsValue::from_str(&fields.timestamp));
    result.push(&JsValue::from_str(&fields.filename));
    result.push(&JsValue::from_str(&fields.author_note));
    Ok(result)
}
