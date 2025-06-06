// ✅ Red5Sorcery - src/lib.rs (UI section fully corrected)

use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::File;
use gloo_timers::callback::{Interval, Timeout};
use web_sys::{
    window, Event, HtmlInputElement, HtmlImageElement, Url
};
use js_sys::Uint8Array;
use sycamore::web::DomNode;
use sycamore::noderef::NodeRef;

use image::{DynamicImage, RgbaImage, Rgba, load_from_memory};
use js_sys::Date;

use image::{ImageError};
use image::error::LimitErrorKind;
use image::ImageOutputFormat;

// Core Image Processing Types
use image::{ImageBuffer};


// JS interop: working with arrays and blobs
use js_sys::{Array};
use web_sys::{
    Blob, BlobPropertyBag, HtmlAnchorElement,
    Window, Document, Element
};

// Regex (if needed)
use regex::Regex;

use wasm_bindgen_futures::JsFuture;

use std::io::BufWriter;
use std::path::Path;

use chrono::Utc;

use base64::engine::general_purpose;
use base64::Engine as _;






#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

        // ✅ Track main app visit immediately
    let document = web_sys::window().unwrap().document().unwrap();
    let iframe = document.create_element("iframe").unwrap();
    iframe.set_attribute("src", "https://red5sorcerystatsmainapp.blogspot.com").unwrap();
    iframe.set_attribute("width", "1").unwrap();
    iframe.set_attribute("height", "1").unwrap();
    iframe.set_attribute("style", "display:none;").unwrap();
    iframe.set_attribute("id", "visit-tracker").unwrap();
    document.body().unwrap().append_child(&iframe).unwrap();

    // Optional: remove it after 2 seconds
    let document_clone = document.clone();
    gloo_timers::callback::Timeout::new(2000, move || {
        if let Some(iframe)= document_clone.get_element_by_id("visit-tracker") {
            iframe.remove();
        }
    }).forget();

    // ✅ Render Sycamore UI
    sycamore::render(|cx| {
        let use_cases = vec![
            "Hide a Zip file in a color image. Two images, with a cloaked zip file, will be downloaded in color and grayscale.",
            "Best practice: store each image in separate locations for the highest security.",
            "You need both the color and grayscale image to extract the cloaked zip file and author notes.",
            "This web app is best suited for desktops, laptops, and tablets. Opera, Chrome, and Edge, are recommended browsers.",
            "A secret love letter. Two images hold everything: words, music, memories—just for them.",
            "A whistleblower hides proof of corruption, split across image fragments—safe until all pieces find the right hands.",
            "A refugee carries vital family records hidden safely inside two photographs.",
            "An artist embeds a banned poem into an album cover, preserving forbidden words for future generations."
        ];
        let len = use_cases.len();

        let idx = create_rc_signal(0);
        let fading = create_rc_signal(false);
        let uncloak_enabled = create_rc_signal(false);

        let secret_message = create_signal(cx, String::new());
        let current_tier = create_signal(cx, String::from("FREE"));
        let status_message = create_rc_signal(String::from("Ready to cloak or decloak."));

        let zip_valid = create_rc_signal(false);
        let zip_valid_view = zip_valid.clone();
        let zip_valid_change = zip_valid.clone();
        let zip_valid_cloak = zip_valid.clone();

        let image_valid = create_rc_signal(false);
        let image_valid_view = image_valid.clone();
        let image_valid_change = image_valid.clone();
        let image_valid_cloak = image_valid.clone();

        let image1_file = create_signal(cx, None::<File>);
        let image2_file = create_signal(cx, None::<File>);

        let _input_ref_1: &NodeRef<DomNode> = create_node_ref(cx);
        let _input_ref_2: &NodeRef<DomNode> = create_node_ref(cx);

        Interval::new(10000, {
            let idx = idx.clone();
            let fading = fading.clone();
            move || {
                fading.set(true);
                Timeout::new(800, {
                    let idx = idx.clone();
                    let fading = fading.clone();
                    move || {
                        idx.set(((*idx.get() + 1) % len));
                        fading.set(false);
                    }
                }).forget();
            }
        }).forget();

        let status_message_for_button = status_message.clone();
        let status_message_for_image1 = status_message.clone();
        let status_message_for_image2 = status_message.clone();


        view! { cx,
            div(class="page") {
                h1(class="title") { "Red5Sorcery" }
                p(class="fine-print", style="margin-top: 0;") { "Some secrets are for your eyes only" }

                div(class="button-group mt-8") {
                    button(
                        class="btn",
                        on:click={ move |_| get_input_by_id("zip-input").click() }
                    ) { "Choose Zip" }

                    input(
                        id="zip-input",
                        type="file",
                        accept=".zip",
                        style="display: none;",
                        on:change={
                            let current_tier = current_tier.clone();
                            let status_message = status_message.clone();
                            let zip_valid = zip_valid_change.clone();
                            move |ev: Event| {
                                if let Some(file) = get_file_from_event(&ev) {
                                    handle_zip_file(file, &current_tier, &zip_valid, &status_message);
                                }
                            }
                        }
                    )

                    button(
                        class="btn ml-4",
                        on:click={ move |_| get_input_by_id("image-input").click() }
                    ) { "Choose Image" }

                    input(
                        id="image-input",
                        type="file",
                        accept=".png,jpg,jpeg",
                        style="display: none;",
                        on:change={
                            let current_tier = current_tier.clone();
                            let status_message = status_message.clone();
                            let image_valid = image_valid_change.clone();
                            move |ev: Event| {
                                if let Some(file) = get_file_from_event(&ev) {
                                    handle_image_file(file, &current_tier, &image_valid, &status_message);
                                }
                            }
                        }
                    )
                }

                div(class="flex justify-center w-full px-4 symmetric-spacing") {
                    input(
                        type="text",
                        placeholder="Author Note (optional, max 100 chars)",
                        maxlength="100",
                        class="author-note-input",
                        bind:value=secret_message
                    )
                }

                div(class="button-group symmetric-spacing") {
                    button(
                        class="btn",
                        prop:disabled = !(*zip_valid_cloak.get() && *image_valid_cloak.get()),
                        on:click={
                            let status_message = status_message.clone();
                            move |_| {
                                status_message.set("Cloaking... this may take seconds to a couple of minutes, based on Zip and image sizes.".to_string());
                                spawn_local_cloak_process(status_message.clone());
                            }
                        }
                    ) { "Cloak" }
                }

                div(class="status-bar") {
                    (status_message.get())
                }

                p(class="fine-print", style="margin-top: 0;") { "Refresh to start again." }

                div(class="button-group mt-12") {
                    button(
                        class="btn",
                        on:click={ move |_| get_input_by_id("image1-input").click() }
                    ) { "Image 1 (color)" }

                    input(
                        id="image1-input",
                        type="file",
                        accept=".png,.jpg,.jpeg",
                        style="display: none;",
                        on:change={
                            let image1_file = image1_file.clone();
                            let image2_file = image2_file.clone();
                            let uncloak_enabled = uncloak_enabled.clone();
                            let status_message_cloned = status_message_for_image1.clone(); // ✅ status bar update

                            move |ev: Event| {
                                if let Some(file) = get_file_from_event(&ev) {
                                    image1_file.set(Some(file));
                                    status_message_cloned.set("Image 1 accepted. Next, choose image 2 if you haven't.".to_string()); // ✅ user feedback
                                    if image2_file.get().is_some() {
                                        uncloak_enabled.set(true);
                                    }
                                }
                            }
                        }

                    )

                    button(
                        class="btn ml-4",
                        on:click={ move |_| get_input_by_id("image2-input").click() }
                    ) { "Image 2 (grayscale)" }

                    input(
                        id="image2-input",
                        type="file",
                        accept=".png,.jpg,.jpeg",
                        style="display: none;",
                        on:change={
                            let image1_file = image1_file.clone();
                            let image2_file = image2_file.clone();
                            let uncloak_enabled = uncloak_enabled.clone();
                            let status_message_cloned = status_message_for_image2.clone(); // ✅ add this

                            move |ev: Event| {
                                if let Some(file) = get_file_from_event(&ev) {
                                    image2_file.set(Some(file));
                                    status_message_cloned.set("Image 2 accepted: Click Decloak if enabled, or choose Image 1.".to_string()); // ✅ status update
                                    if image1_file.get().is_some() {
                                        uncloak_enabled.set(true);
                                    }
                                }
                            }
                        }
                    )
                }


                div(class="button-group mt-4") {
                    button(
                        class="btn",
                        prop:disabled = !*uncloak_enabled.get(),
                        on:click={
                            let image1_file = image1_file;
                            let image2_file = image2_file;
                            let status_message_cloned = status_message_for_button.clone(); // ✅ all good now

                            move |_| {
                                if image1_file.get().is_some() && image2_file.get().is_some() {
                                    status_message_cloned.set("Decloaking in progress...".to_string());
                                    spawn_local_uncloak_process(
                                        image1_file,
                                        image2_file,
                                        status_message_cloned.clone(),
                                    );
                                }
                            }
                        }
                    ) {
                        "Decloak"
                    }
                }



                input(
                    id="unlock-png-input",
                    type="file",
                    accept=".png",
                    style="display: none;",
                    on:change={
                        let current_tier = current_tier.clone();
                        move |ev: Event| {
                            if let Some(file) = get_file_from_event(&ev) {
                                handle_unlock_file(&file, &current_tier);
                            }
                        }
                    }
                )

                div(class="flex justify-center mt-12") {
                    button(
                        class="unlock-button",
                        prop:disabled = *current_tier.get() != "FREE",
                        on:click={ move |_| {
                            get_input_by_id("unlock-png-input").click();
                        }}
                    ) {
                        (match current_tier.get().as_str() {
                            "PRO" => "PRO Unlocked",
                            "VEGAS" => "VEGAS Unlocked",
                            _ => "Unlock PRO or VEGAS"
                        })
                    }
                    p(class="fine-print text-center", style="margin-top: 0.75rem;") {
                        (match current_tier.get().as_str() {
                            "PRO"   => "Cloaking max Zip file size: 20 MB",
                            "VEGAS" => "Cloaking max Zip file size: 40 MB",
                            _       => "Cloaking max Zip file size: 6 MB"
                        })
                    }

                    a(href="https://TheSocialCredit.etsy.com", target="_blank", style="color:inherit; text-decoration:underline;") {
                        "Buy unlock images at TheSocialCredit.etsy.com"
                    }

                }

                footer(class="fine-print text-center mt-12") {
                    "Powered by RQSM"
                    br() "(Red5Sorcery Quantum Security Model)"
                    br() "A Canadian Innovation"
                    br()
                    a(href="https://red5sorcery-termsofuse.blogspot.com/2025/05/terms-of-use-and-privacy-notice.html", target="_blank", style="color:inherit; text-decoration:underline;") {
                        "Terms of Use"
                    }
                    br() "Buy the book and source code (coming soon)"
                    a(href="https://red5sorceryhelp.blogspot.com", target="_blank", style="color:inherit; text-decoration:underline; display:block; margin-top:1rem;") {
                        "Help"
                    }
                }

                div(class="use-case-card mt-12") {
                    div(class=if *fading.get() { "use-case-text fadeout" } else { "use-case-text" }) {
                        (use_cases[*idx.get()])
                    }
                }
            }
        }
    });
}


// Helpers, largely unchanged :

fn get_input_by_id(id: &str) -> HtmlInputElement {
    window().unwrap().document().unwrap()
        .get_element_by_id(id).unwrap()
        .dyn_into().unwrap()
}

fn get_file_from_event(ev: &Event) -> Option<web_sys::File> {
    let input: HtmlInputElement = ev.target().unwrap().unchecked_into();
    input.files().unwrap().get(0)
}

fn handle_zip_file(
    file: web_sys::File,
    current_tier: &Signal<String>,
    zip_valid: &RcSignal<bool>,
    status_message: &RcSignal<String>,
) {
    let size = file.size();

    // ←– Change these three lines to your new limits (in bytes)
    let (tier_limit, readable_limit) = match current_tier.get().as_str() {
        "VEGAS" => (40_000_000.0, "40 MB"),   // e.g. VEGAS → 40 MB
        "PRO"   => (20_000_000.0, "20 MB"),   // e.g. PRO → 20 MB
        _       => (6_000_000.0,  "6 MB"),    // e.g. FREE → 6 MB
    };

    if size > tier_limit {
        zip_valid.set(false);
        status_message.set(format!("Please choose a .zip file ≤ {}", readable_limit));
    } else {
        zip_valid.set(true);
        status_message.set("Zip file accepted. Next, choose an image if you haven't.".to_string());
    }
}


fn handle_image_file(
    file: web_sys::File,
    current_tier: &Signal<String>,
    image_valid: &RcSignal<bool>,
    status_message: &RcSignal<String>,
) {
    let img = HtmlImageElement::new().unwrap();
    let img_clone = img.clone();

    let image_valid = image_valid.clone();
    let status_message = status_message.clone();
    let tier_val = current_tier.get().to_string(); // capture tier as owned string

    let onload = Closure::wrap(Box::new(move || {
        let (min_w, min_h) = match tier_val.as_str() {
            "PRO" => (5312, 7082),
            "VEGAS" => (6144, 8192),
            _ => (3072, 4096),
        };

        let width = img_clone.width();
        let height = img_clone.height();

        if width < min_w || height < min_h {
            image_valid.set(false);
            status_message.set(format!(
                "Too small: {}×{}. Minimum required: {}×{}.",
                width, height, min_w, min_h
            ));
        } else {
            image_valid.set(true);
            status_message.set(format!(
                "Image accepted: Click Cloak if enabled, or choose a Zip to cloak.",
            ));
        }
    }) as Box<dyn FnMut()>);

    img.set_onload(Some(onload.as_ref().unchecked_ref()));

    // ✅ Ensure onload fires by revoking old URL (browser may cache otherwise)
    let old_url = img.src();
    if !old_url.is_empty() {
        Url::revoke_object_url(&old_url).ok();
    }

    let new_url = Url::create_object_url_with_blob(&file).unwrap();
    img.set_src(&new_url);

    onload.forget(); // prevent memory leak
}


fn handle_unlock_file(
    file: &web_sys::File,
    current_tier: &Signal<String>,
) {
    let name = file.name();
    let size = file.size();
    let tier = if name == "PRO_Unlock.png" && size == 957_537.0 {
        "PRO"
    } else if name == "VEGAS_Unlock.png" && size == 12_674.0 {
        "VEGAS"
    } else {
        "FREE"
    };
    current_tier.set(tier.to_string());
    web_sys::console::log_1(&format!("{} Tier Unlocked", tier).into());
}


/// Pads a string to 100 bytes.
/// - `pad_right = true`: right-pad with spaces
/// - `pad_right = false`: left-pad with zeroes
fn pad_to_100(input: &str, pad_right: bool) -> Vec<u8> {
    let mut bytes = input.as_bytes().to_vec();
    if bytes.len() > 100 {
        bytes.truncate(100);
    }
    if pad_right {
        bytes.resize(100, b' ');
    } else {
        while bytes.len() < 100 {
            bytes.insert(0, b'0');
        }
    }
    bytes
}


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


fn spawn_local_cloak_process(status_message: RcSignal<String>) {
    spawn_local(async move {
        use wasm_bindgen::JsCast;
        use web_sys::{window, HtmlInputElement};
        use js_sys::Uint8Array;
        use base64::engine::general_purpose;
        use base64::Engine as _;

        let window = window().unwrap();
        let document = window.document().unwrap();

        // --- Retrieve ZIP File ---
        let zip_input = document.get_element_by_id("zip-input").unwrap()
            .dyn_into::<HtmlInputElement>().unwrap();
        let zip_file = zip_input.files().unwrap().get(0).unwrap();
        let zip_name = zip_file.name();

        let zip_array_buffer = wasm_bindgen_futures::JsFuture::from(zip_file.array_buffer()).await.unwrap();
        let zip_bytes = Uint8Array::new(&zip_array_buffer).to_vec();

        // ✅ Base64 encode ZIP bytes to match CLI behavior
        let base64_str = general_purpose::STANDARD.encode(&zip_bytes);
        let base64_zip_bytes = base64_str.as_bytes();

        // --- Retrieve Image File ---
        let img_input = document.get_element_by_id("image-input").unwrap()
            .dyn_into::<HtmlInputElement>().unwrap();
        let img_file = img_input.files().unwrap().get(0).unwrap();

        let image_name = img_file.name();
        let base_image_name = image_name
            .strip_suffix(".png")
            .or_else(|| image_name.strip_suffix(".jpg"))
            .or_else(|| image_name.strip_suffix(".jpeg"))
            .unwrap_or(&image_name);

        let img_buffer = wasm_bindgen_futures::JsFuture::from(img_file.array_buffer()).await.unwrap();
        let img_u8 = Uint8Array::new(&img_buffer).to_vec();

        let base_image = match load_from_memory(&img_u8) {
            Ok(img) => img,
            Err(e) => {
                status_message.set(format!("Failed to load image: {}", e));
                return;
            }
        };

        // --- Author note ---
        let author_note_input = document.query_selector(".author-note-input").unwrap().unwrap();
        let note_input = author_note_input.dyn_into::<HtmlInputElement>().unwrap();
        let author_note = note_input.value();

        // --- Cloak ---
        match cloak_zip_to_images(&base_image, &zip_name, base64_zip_bytes, &author_note) {
            Ok((img1, img2)) => {
                // Download both images
                download_rgba_image(img1, &format!("{}_color.png", base_image_name));
                download_rgba_image(img2, &format!("{}_grayscale.png", base_image_name));

                status_message.set("Check your downloads folder for two PNG images in color and grayscale.".to_string());

                // ✅ Blogger iFrame tracking
                let iframe = document.create_element("iframe").unwrap();
                iframe.set_attribute("src", "https://red5sorcerycloakingstats.blogspot.com").unwrap();
                iframe.set_attribute("width", "1").unwrap();
                iframe.set_attribute("height", "1").unwrap();
                iframe.set_attribute("style", "display:none;").unwrap();
                iframe.set_attribute("id", "cloak-tracker").unwrap();

                document.body().unwrap().append_child(&iframe).unwrap();

                let document_clone = document.clone();
                gloo_timers::callback::Timeout::new(2000, move || {
                    if let Some(existing_iframe) = document_clone.get_element_by_id("cloak-tracker") {
                        existing_iframe.remove();
                    }
                }).forget();
            }
            Err(e) => {
                status_message.set(format!("Cloak failed: {}", e));
            }
        }
    });
}




fn download_rgba_image(img: RgbaImage, filename: &str) {
    use web_sys::{Url, Blob, BlobPropertyBag};
    use wasm_bindgen::JsCast;
    use js_sys::Uint8Array;
    use image::ImageOutputFormat;

    let mut png_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageOutputFormat::Png).unwrap();

    let array = Uint8Array::from(&png_bytes[..]);
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array.buffer());

    let blob = Blob::new_with_u8_array_sequence_and_options(
        &blob_parts,
        BlobPropertyBag::new().type_("image/png"),
    ).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let a = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    Url::revoke_object_url(&url).unwrap();
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


pub fn download_zip(bytes: &[u8], timestamp: &str, filename: &str) {
    let full_filename = format!("{}_{}", timestamp, filename);

    // Create a JS Uint8Array from the Rust byte slice
    let uint8_array = js_sys::Uint8Array::from(bytes);

    // Create a Blob
    let array: Array = Array::new();
    array.push(&uint8_array.buffer());

    let mut blob_props = BlobPropertyBag::new();
    blob_props.type_("application/zip");

    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &blob_props)
        .expect("Failed to create Blob");

    let url = Url::create_object_url_with_blob(&blob).expect("Failed to create object URL");

    // Create a link and trigger download
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document");

    let a = document
        .create_element("a")
        .unwrap()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();

    a.set_href(&url);
    a.set_download(&full_filename);
    a.click();

    // Cleanup
    Url::revoke_object_url(&url).expect("Failed to revoke object URL");
}


pub fn download_author_note(note: &str, timestamp: &str) {
    let filename = format!("{}_UTC_author_note.txt", timestamp);

    // Convert the note to a JS Uint8Array
    let note_bytes = note.as_bytes();
    let uint8_array = js_sys::Uint8Array::from(note_bytes);

    // Create a Blob
    let array: Array = Array::new();
    array.push(&uint8_array.buffer());

    let mut blob_props = BlobPropertyBag::new();
    blob_props.type_("text/plain");

    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &blob_props)
        .expect("Failed to create Blob");

    let url = Url::create_object_url_with_blob(&blob).expect("Failed to create object URL");

    // Create a link and trigger download
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document");

    let a = document
        .create_element("a")
        .unwrap()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();

    a.set_href(&url);
    a.set_download(&filename);
    a.click();

    // Cleanup
    Url::revoke_object_url(&url).expect("Failed to revoke object URL");
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




fn spawn_local_uncloak_process(
    image1_file: &Signal<Option<File>>,
    image2_file: &Signal<Option<File>>,
    status_message: RcSignal<String>,
) {
    let file1_opt = image1_file.get();
    let file2_opt = image2_file.get();
    let status_message_cloned = status_message.clone();

    spawn_local(async move {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use js_sys::Uint8Array;
        use image::{load_from_memory, RgbaImage};
        use base64::engine::general_purpose;
        use base64::Engine as _;

        let window = window().unwrap();
        let document = window.document().unwrap();

        // --- Image file validation ---
        let file1 = match file1_opt.as_ref() {
            Some(f) => f.clone(),
            None => {
                status_message_cloned.set("❌ Image 1 not selected.".to_string());
                return;
            }
        };
        let file2 = match file2_opt.as_ref() {
            Some(f) => f.clone(),
            None => {
                status_message_cloned.set("❌ Image 2 not selected.".to_string());
                return;
            }
        };

        // --- Load files into memory ---
        let buffer1 = match JsFuture::from(file1.array_buffer()).await {
            Ok(buf) => buf,
            Err(_) => {
                status_message_cloned.set("❌ Failed to read Image 1.".to_string());
                return;
            }
        };
        let buffer2 = match JsFuture::from(file2.array_buffer()).await {
            Ok(buf) => buf,
            Err(_) => {
                status_message_cloned.set("❌ Failed to read Image 2.".to_string());
                return;
            }
        };
        let bytes1 = Uint8Array::new(&buffer1).to_vec();
        let bytes2 = Uint8Array::new(&buffer2).to_vec();

        let img1: RgbaImage = match load_from_memory(&bytes1) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                status_message_cloned.set(format!("❌ Image 1 failed to load: {}", e));
                return;
            }
        };
        let img2: RgbaImage = match load_from_memory(&bytes2) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                status_message_cloned.set(format!("❌ Image 2 failed to load: {}", e));
                return;
            }
        };

        // --- Ensure dimensions match ---
        let (w1, h1) = img1.dimensions();
        let (w2, h2) = img2.dimensions();
        if w1 != w2 || h1 != h2 {
            status_message_cloned.set(format!(
                "❌ Cannot Decloak: image sizes differ ({}×{} vs {}×{}).",
                w1, h1, w2, h2
            ));
            return;
        }

        // --- Reconstruct composite and extract ---
        let composite = reconstruct_composite_image(&img1, &img2);
        let (_anchor_index, shuffled_indices, _seed_string) =
            compute_anchor_seed_and_shuffle(&composite);

        let header_bits: Vec<u8> = extract_bits_lsb(&composite, 400, &shuffled_indices);
        let fields = match parse_and_validate_header(&header_bits) {
            Ok(f) => f,
            Err(e) => {
                status_message_cloned.set(format!("❌ Header error: {}", e));
                return;
            }
        };

        let total_bytes_needed = 400 + fields.payload_size;
        let full_stream = extract_bits_lsb(&composite, total_bytes_needed, &shuffled_indices);
        let base64_payload = &full_stream[400..(400 + fields.payload_size)];

        let decoded_zip: Vec<u8> = match general_purpose::STANDARD.decode(base64_payload) {
            Ok(data) => data,
            Err(e) => {
                status_message_cloned.set(format!("❌ Failed to decode Base64 payload: {}", e));
                return;
            }
        };

        download_zip(&decoded_zip, &fields.timestamp, &fields.filename);
        download_author_note(&fields.author_note, &fields.timestamp);

        status_message_cloned.set(format!(
            "✅ Decloaked. Files saved with UTC timestamp: {}",
            &fields.timestamp
        ));

        // ✅ Blogger iFrame tracking for decloaking
        let iframe = document.create_element("iframe").unwrap();
        iframe.set_attribute("src", "https://red5sorcerydecloakingstats.blogspot.com").unwrap();
        iframe.set_attribute("width", "1").unwrap();
        iframe.set_attribute("height", "1").unwrap();
        iframe.set_attribute("style", "display:none;").unwrap();
        iframe.set_attribute("id", "decloak-tracker").unwrap();

        document.body().unwrap().append_child(&iframe).unwrap();

        let document_clone = document.clone();
        gloo_timers::callback::Timeout::new(2000, move || {
            if let Some(existing_iframe) = document_clone.get_element_by_id("decloak-tracker") {
                existing_iframe.remove();
            }
        }).forget();
    });
}












fn download_seed_log(seed_string: &str) {
    let content = format!("Red5Sorcery Seed\n=================\n{}\n", seed_string);
    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(&content));

    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        BlobPropertyBag::new().type_("text/plain"),
    ).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();

    let document = window().unwrap().document().unwrap();
    let a = document.create_element("a").unwrap().dyn_into::<HtmlAnchorElement>().unwrap();
    a.set_href(&url);
    a.set_download("Red5Sorcery_Seed.txt");
    a.click();

    Url::revoke_object_url(&url).unwrap();
}


fn download_header_debug(filename: &str, header_bytes: &[u8]) {
    let hex_preview: Vec<String> = header_bytes.iter().map(|b| format!("{:02X}", b)).collect();
    let text_preview = String::from_utf8_lossy(header_bytes);

    let content = format!(
        "Red5Sorcery Header Debug\n==========================\n\
        Bytes (Hex):\n{}\n\nAs Text:\n{}\n",
        hex_preview.join(" "),
        text_preview
    );

    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(&content));

    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        BlobPropertyBag::new().type_("text/plain"),
    ).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let document = window().unwrap().document().unwrap();
    let a = document.create_element("a").unwrap().dyn_into::<HtmlAnchorElement>().unwrap();
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    Url::revoke_object_url(&url).unwrap();
}


fn download_shuffle_debug(filename: &str, count: usize, context: &str) {
    let content = format!(
        "Red5Sorcery Shuffle Debug\n===========================\n\
        Context: {}\n\
        Shuffled Indices Count: {}\n\
        Minimum Required for Header: 800\n",
        context,
        count
    );

    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(&content));

    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        BlobPropertyBag::new().type_("text/plain"),
    ).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let document = window().unwrap().document().unwrap();
    let a = document.create_element("a").unwrap().dyn_into::<HtmlAnchorElement>().unwrap();
    a.set_href(&url);
    a.set_download(filename);
    a.click();

    Url::revoke_object_url(&url).unwrap();
}


fn download_header_first8_debug(header_bytes: &[u8]) {
    let hex: Vec<String> = header_bytes.iter().take(8).map(|b| format!("{:02X}", b)).collect();
    let dec: Vec<String> = header_bytes.iter().take(8).map(|b| format!("{}", b)).collect();
    let bin: Vec<String> = header_bytes.iter().take(8).map(|b| format!("{:08b}", b)).collect();

    let content = format!(
        "Red5Sorcery Header First 8 Bytes\n==================================\n\
        Hex: {}\n\
        Dec: {}\n\
        Bin:\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        hex.join(" "),
        dec.join(" "),
        bin[0], bin[1], bin[2], bin[3],
        bin[4], bin[5], bin[6], bin[7]
    );

    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(&content));

    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        BlobPropertyBag::new().type_("text/plain"),
    ).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let document = window().unwrap().document().unwrap();
    let a = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
    a.set_href(&url);
    a.set_download("Red5Sorcery_Uncloak_Header_First8.txt");
    a.click();
    Url::revoke_object_url(&url).unwrap();
}


fn download_raw_timestamp_check(header: &[u8]) {
    let ts_raw = &header[0..100];
    let as_string = String::from_utf8_lossy(ts_raw);
    let hex = ts_raw.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
    let visible = ts_raw.iter().map(|b| if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' }).collect::<String>();

    let content = format!(
        "Timestamp Field (First 100 bytes)\n===============================\n\
         UTF-8 Preview: {}\n\
         Visible ASCII: {}\n\
         Hex: {}\n",
        as_string, visible, hex
    );

    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(&content));

    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        BlobPropertyBag::new().type_("text/plain"),
    ).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let document = window().unwrap().document().unwrap();
    let a = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
    a.set_href(&url);
    a.set_download("Red5Sorcery_TimestampField_Debug.txt");
    a.click();
    Url::revoke_object_url(&url).unwrap();
}


fn download_bitstream_debug(label: &str, bytes: &[u8]) {
    let bits: Vec<String> = bytes
        .iter()
        .flat_map(|byte| (0..8).map(move |i| format!("{}", (byte >> i) & 1)))
        .collect();

    let hex: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    let ascii: String = bytes
        .iter()
        .map(|b| if b.is_ascii_graphic() { *b as char } else { '.' })
        .collect();

    let content = format!(
        "Red5Sorcery Bitstream Debug: {}\n==============================\n\
        Bytes (Hex): {}\n\
        ASCII Preview: {}\n\
        Bits (LSB First):\n{}\n",
        label,
        hex.join(" "),
        ascii,
        bits.chunks(8).map(|chunk| chunk.join(" ")).collect::<Vec<_>>().join("\n")
    );

    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(&content));

    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        BlobPropertyBag::new().type_("text/plain"),
    ).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let document = window().unwrap().document().unwrap();
    let a = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
    a.set_href(&url);
    a.set_download(&format!("Red5Sorcery_{}_Bits.txt", label));
    a.click();
    Url::revoke_object_url(&url).unwrap();
}



fn save_debug_image(img: &RgbaImage, filename: &str) {
    use web_sys::{Url, Blob, BlobPropertyBag};
    use wasm_bindgen::JsCast;
    use js_sys::Uint8Array;
    use image::ImageOutputFormat;

    let mut png_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageOutputFormat::Png).unwrap();

    let array = Uint8Array::from(&png_bytes[..]);
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array.buffer());

    let blob = Blob::new_with_u8_array_sequence_and_options(
        &blob_parts,
        BlobPropertyBag::new().type_("image/png"),
    ).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let a = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    Url::revoke_object_url(&url).unwrap();
}

