use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Grabing the whole web document 
    let document = web_sys::window().unwrap().document().unwrap();
    // Get the canvas 
    let canvas = document.get_element_by_id("canvas").unwrap();
    // Shadow canvas (get the html canvas element)
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    // init webgl
    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    // create the vertex shader
    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        r#"
        precision mediump float;
        attribute vec2 vertPosition;
        attribute vec3 vertColor; 
        varying vec3 fragColor;
        void main()
        {
            fragColor = vertColor;
            gl_Position = vec4(vertPosition, 0.0, 1.0);
        } 
    "#,
    )?;
    
    // create the fragment shader
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
        precision mediump float;
        varying vec3 fragColor; 
        void main()
        {
            gl_FragColor = vec4(fragColor, 1.0);
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
    let vertices: [f32; 15] = 
        [ // X, Y,       R, G, B
            0.0, 0.5,    1.0, 1.0, 0.0,
            -0.5, -0.5,  0.7, 0.0, 1.0,
            0.5, -0.5,   0.1, 1.0, 0.6
        ];
   
    // webGL needs a buffer for the triangle 
    let triangle_buffer = context.create_buffer().ok_or("failed to create buffer")?; 
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&triangle_buffer)); 
 
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
        let vert_array = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
    // Think we're safe now 
    
    let pos_attrib_loc = context.get_attrib_location(&program, "vertPosition") as u32;
    let color_attrib_loc = context.get_attrib_location(&program, "vertColor") as u32;
    
    context.vertex_attrib_pointer_with_i32(pos_attrib_loc, // Attribute location
                                           2, // Number of elements per attribute  
                                           WebGlRenderingContext::FLOAT, // Type of element
                                           false, // Is this data normalized  
                                           5 * std::mem::size_of::<f32>() as i32, // Size of individual vertex 
                                           0); // Offset from the beginning of a single vertex to this attribute 
    context.vertex_attrib_pointer_with_i32(color_attrib_loc, // Attribute location
                                           3, // Number of elements per attribute  
                                           WebGlRenderingContext::FLOAT, // Type of element
                                           false, // Is this data normalized  
                                           5 * std::mem::size_of::<f32>() as i32, // Size of individual vertex 
                                           2 * std::mem::size_of::<f32>() as i32); // Offset from the beginning of a single vertex to this attribute
    
    context.enable_vertex_attrib_array(pos_attrib_loc);
    context.enable_vertex_attrib_array(color_attrib_loc);

    context.clear_color(0.75, 0.85, 0.8, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);
    
    // okay now draw the thing
    context.draw_arrays(
        WebGlRenderingContext::TRIANGLES, // Mode
        0, // First Index
        3, // count 
    );
    Ok(())
}

pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
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
    context: &WebGlRenderingContext,
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
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
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
