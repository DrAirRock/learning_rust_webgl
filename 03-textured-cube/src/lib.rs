use std::rc::Rc; 
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext as GL, WebGlShader};
use gl_matrix; 

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Grabing the whole web document 
    let document = document();
    // Get the canvas 
    let canvas = document.get_element_by_id("game-surface").unwrap();
    // Shadow canvas (get the html canvas element)
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    // grab image 
    let image = document.get_element_by_id("crate-image").unwrap();
    // shadow image element
    let image: web_sys::HtmlImageElement = image.dyn_into::<web_sys::HtmlImageElement>()?;
    // grab window prformance 
    let performance = window().performance().unwrap();

    // init webgl
    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<GL>()?; 
   
    context.clear_color(0.75, 0.85, 0.8, 1.0);
    context.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
    context.enable(GL::DEPTH_TEST);
    context.enable(GL::CULL_FACE);
    context.front_face(GL::CCW);
    context.cull_face(GL::BACK);
 
    // create the vertex shader
    let vert_shader = compile_shader(
        &context,
        GL::VERTEX_SHADER,
        r#"
        precision mediump float; 

        attribute vec3 vertPosition;
        attribute vec2 vertTexCoord;
        varying vec2 fragTexCoord;
        uniform mat4 mWorld;
        uniform mat4 mView;
        uniform mat4 mProj;

        void main()
        {
            fragTexCoord = vertTexCoord;
            gl_Position = mProj * mView * mWorld * vec4(vertPosition, 1.0);
        }
        "#,
    )?;
    
    // create the fragment shader
    let frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        r#"
        precision mediump float;

        varying vec2 fragTexCoord; 
        uniform sampler2D sampler; 

        void main()
        {
            gl_FragColor = texture2D(sampler, fragTexCoord);
        }
    "#,
    )?;
    // now that the shaders have been compiled we need to link the program 
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    //
    // create the buffer
    // expects it as f32 
    //
    let box_vertices: [f32; 120] = 
	[ // X, Y, Z           U, V
		// Top
		-1.0, 1.0, -1.0,   0., 0.,
		-1.0, 1.0, 1.0,    0., 1.,
		1.0, 1.0, 1.0,     1., 1.,
		1.0, 1.0, -1.0,    1., 0.,

		// Left
		-1.0, 1.0, 1.0,    0., 0.,
		-1.0, -1.0, 1.0,   1., 0.,
		-1.0, -1.0, -1.0,  1., 1.,
		-1.0, 1.0, -1.0,   0., 1.,

		// Right
		1.0, 1.0, 1.0,    1., 1.,
		1.0, -1.0, 1.0,   0., 1.,
		1.0, -1.0, -1.0,  0., 0.,
		1.0, 1.0, -1.0,   1., 0.,

		// Front
		1.0, 1.0, 1.0,    1., 1.,
		1.0, -1.0, 1.0,    1., 0.,
		-1.0, -1.0, 1.0,    0., 0.,
		-1.0, 1.0, 1.0,    0., 1.,

		// Back
		1.0, 1.0, -1.0,    0., 0.,
		1.0, -1.0, -1.0,    0., 1.,
		-1.0, -1.0, -1.0,    1., 1.,
		-1.0, 1.0, -1.0,    1., 0.,

		// Bottom
		-1.0, -1.0, -1.0,   1., 1.,
		-1.0, -1.0, 1.0,    1., 0.,
		1.0, -1.0, 1.0,     0., 0.,
		1.0, -1.0, -1.0,    0., 1.
	];

    let box_indices: [u16; 36] =
	[
		// Top
		0, 1, 2,
		0, 2, 3,

		// Left
		5, 4, 6,
		6, 4, 7,

		// Right
		8, 9, 10,
		8, 10, 11,

		// Front
		13, 12, 14,
		15, 14, 12,

		// Back
		16, 17, 18,
		16, 18, 19,

		// Bottom
		21, 20, 22,
		22, 20, 23
	];

    // webGL needs a buffer for the box
    let box_vertex_buffer = context.create_buffer().ok_or("failed to create buffer")?; 
    context.bind_buffer(GL::ARRAY_BUFFER, Some(&box_vertex_buffer)); 
 
    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe { 
        // this is the unsafe... 
        let vert_array = js_sys::Float32Array::view(&box_vertices);

        context.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &vert_array,
            GL::STATIC_DRAW,
        );
    }
    // Think we're safe now 
    
    // webGL needs a buffer for the box
    let box_index_buffer = context.create_buffer().ok_or("failed to create buffer")?; 
    context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&box_index_buffer)); 
 
    // Note that `Uint16Array::view` is also somewhat dangerous
    unsafe { 
        // this is the unsafe... 
        let ind_array = js_sys::Uint16Array::view(&box_indices);

        context.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &ind_array,
            GL::STATIC_DRAW,
        );
    } 
    // Think we're safe now 
    
    let pos_attrib_loc = context.get_attrib_location(&program, "vertPosition") as u32;
    let tex_coord_attrib_loc = context.get_attrib_location(&program, "vertTexCoord") as u32;
    context.vertex_attrib_pointer_with_i32(pos_attrib_loc, // Attribute location
                                           3, // Number of elements per attribute  
                                           GL::FLOAT, // Type of element
                                           false, // Is this data normalized  
                                           5 * std::mem::size_of::<f32>() as i32, // Size of individual vertex 
                                           0); // Offset from the beginning of a single vertex to this attribute 
    context.vertex_attrib_pointer_with_i32(tex_coord_attrib_loc, // Attribute location
                                           2, // Number of elements per attribute  
                                           GL::FLOAT, // Type of element
                                           false, // Is this data normalized  
                                           5 * std::mem::size_of::<f32>() as i32, // Size of individual vertex 
                                           3 * std::mem::size_of::<f32>() as i32); // Offset from the beginning of a single vertex to this attribute
    
    context.enable_vertex_attrib_array(pos_attrib_loc);
    context.enable_vertex_attrib_array(tex_coord_attrib_loc);

    // Create Texture 
    let box_texture = context.create_texture();
	context.bind_texture(GL::TEXTURE_2D, box_texture.as_ref());
	context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
	context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
	context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
    context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
    // I don't know why they picked this to be the name 
	context.tex_image_2d_with_u32_and_u32_and_image(
        GL::TEXTURE_2D, 
        0, 
        GL::RGBA as i32, 
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        &image
	)?;
	context.bind_texture(GL::TEXTURE_2D, None);
    
    let mat_world_uniform_location = context.get_uniform_location(&program, "mWorld");
	let mat_view_uniform_location = context.get_uniform_location(&program, "mView");
	let mat_proj_uniform_location = context.get_uniform_location(&program, "mProj");
   
    let mut world_matrix: [f32; 16] = [0.; 16];
	let mut view_matrix: [f32; 16] = [0.; 16];
	let mut proj_matrix: [f32; 16] = [0.; 16];
    gl_matrix::mat4::identity(&mut world_matrix);
    gl_matrix::mat4::look_at(&mut view_matrix, &[0., 0., -8.], &[0., 0., 0.], &[0., 1., 0.]);
    gl_matrix::mat4::perspective(&mut proj_matrix, gl_matrix::common::to_radian(45.), (canvas.client_width() / canvas.client_height()) as f32, 0.1, Some(1000.0));

    context.uniform_matrix4fv_with_f32_array(mat_world_uniform_location.as_ref(), false, &world_matrix);
	context.uniform_matrix4fv_with_f32_array(mat_view_uniform_location.as_ref(), false, &view_matrix);
	context.uniform_matrix4fv_with_f32_array(mat_proj_uniform_location.as_ref(), false, &proj_matrix);

    let mut x_rotation_matrix: [f32; 16] = [0.; 16]; 
    let mut y_rotation_matrix: [f32; 16] = [0.; 16]; 
   
    let mut identity_matrix:[f32; 16] = [0.; 16];
	gl_matrix::mat4::identity(&mut identity_matrix);
    let mut angle: f32 = 0.; 
    
    let f = Rc::new(RefCell::new(None));
    let g = f.clone(); 

    // our event loop 
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || { 
        angle = performance.now() as f32 / 1000_f32 / 6_f32 * 2_f32 * gl_matrix::common::PI;
        gl_matrix::mat4::rotate(&mut y_rotation_matrix, &identity_matrix, angle, &[0., 1., 0.]);
        gl_matrix::mat4::rotate(&mut x_rotation_matrix, &identity_matrix, angle / 4_f32, &[1., 0., 0.]);
        gl_matrix::mat4::mul(&mut world_matrix, &y_rotation_matrix, &x_rotation_matrix);
        context.uniform_matrix4fv_with_f32_array(mat_world_uniform_location.as_ref(), false, &world_matrix);

        context.clear_color(0.75, 0.85, 0.8, 1.0);
        context.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
 
        context.bind_texture(GL::TEXTURE_2D, box_texture.as_ref());
        context.active_texture(GL::TEXTURE0);

        context.draw_elements_with_f64(
                        GL::TRIANGLES, 
                        box_indices.len() as i32,
                        GL::UNSIGNED_SHORT, 
                        0.
                        );
         
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

pub fn compile_shader(
    context: &GL,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &GL,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
