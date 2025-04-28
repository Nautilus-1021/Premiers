use glow::{HasContext, COMPUTE_SHADER, DYNAMIC_COPY, MAP_READ_BIT, SHADER_STORAGE_BARRIER_BIT, SHADER_STORAGE_BUFFER};

const SHADER_CODE: &str = "
#version 430 core

layout (local_size_x = 32, local_size_y = 1, local_size_z = 1) in;
layout (std430, binding = 0) buffer primes_buf {
    unsigned int buf[];
} bufbuf;

void main() {
    unsigned int actslot = gl_WorkGroupID.x;
    unsigned int actnmb = gl_GlobalInvocationID.x * 2 - 1;

    for (unsigned int idx = 3; idx <= sqrt(actnmb); idx += 2) {
        if (actnmb % idx == 0) {
            return;
        }
    }

    atomicOr(bufbuf.buf[actslot], 1 << gl_LocalInvocationID.x);
}";
const BUF_LEN: usize = 235_000;

pub fn compile_shader<CTX: HasContext>(ctx: &CTX) -> Result<(), String> {
    unsafe {
        let shader = ctx.create_shader(COMPUTE_SHADER)?;
        ctx.shader_source(shader, SHADER_CODE);
        ctx.compile_shader(shader);

        if !ctx.get_shader_compile_status(shader) {
            return Err(ctx.get_shader_info_log(shader))
        }

        let program = ctx.create_program()?;
        ctx.attach_shader(program, shader);
        ctx.link_program(program);

        if !ctx.get_program_link_status(program) {
            return Err(ctx.get_program_info_log(program))
        }

        ctx.delete_shader(shader);

        let t2 = std::time::Instant::now();
        let buf = ctx.create_buffer()?;
        ctx.bind_buffer(SHADER_STORAGE_BUFFER, Some(buf));
        ctx.buffer_data_size(SHADER_STORAGE_BUFFER, (size_of::<u32>() * BUF_LEN) as i32, DYNAMIC_COPY);

        ctx.use_program(Some(program));
        ctx.bind_buffer_base(SHADER_STORAGE_BUFFER, 0, Some(buf));

        let t1 = std::time::Instant::now();
        ctx.dispatch_compute(BUF_LEN as u32, 1, 1);
        ctx.memory_barrier(SHADER_STORAGE_BARRIER_BIT);
        ctx.finish();

        let tt = t1.elapsed();
        let tt2 = t2.elapsed();

        ctx.bind_buffer(SHADER_STORAGE_BUFFER, Some(buf));
        let ptr = ctx.map_buffer_range(SHADER_STORAGE_BUFFER, 0, (size_of::<u32>() * BUF_LEN) as i32, MAP_READ_BIT) as *const u32;

        println!("\nRésultats:");

        for iddx in 0..15 {
            let act_nmb = *(ptr.add(iddx));
            for idx in 0..32 {
                if (act_nmb >> idx) & 1 == 1 {
                    print!("{} ", (idx * 2 + iddx * 64) - 1);
                }
            }
        }
        print!("... ");

        let act_nmb = *(ptr.add(BUF_LEN - 1));
        for idx in 0..32 {
            if (act_nmb >> idx) & 1 == 1 {
                print!("{} ", (idx * 2 + (BUF_LEN - 1) * 64) - 1);
            }
        }
        println!();

        let mut act_slot = 0;
        let mut act_idx = 0;
        let mut cpt = 0;
        while act_slot < BUF_LEN {
            if *(ptr.add(act_slot)) >> act_idx & 1 == 1 {
                cpt += 1;
            }

            act_idx += 1;
            if act_idx >= 32 {
                act_idx = 0;
                act_slot += 1;
            }
        }

        println!("Nombre de nombres premiers: {cpt}");

        ctx.unmap_buffer(SHADER_STORAGE_BUFFER);

        println!("\nTemps:\nCalcul: {}s\n+Allocation: {}s", tt.as_secs_f64(), tt2.as_secs_f64());

        Ok(())
    }
}