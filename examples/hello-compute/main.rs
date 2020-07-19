use std::{convert::TryInto, str::FromStr};

async fn run() {
    let numbers = if std::env::args().len() <= 1 {
        let default = vec![1, 2, 3, 4];
        println!("No numbers were provided, defaulting to {:?}", default);
        default
    } else {
        std::env::args()
            .skip(1)
            .map(|s| u32::from_str(&s).expect("You must pass a list of positive integers!"))
            .collect()
    };

    let times = execute_gpu(numbers).await;
    println!("Times: {:?}", times);
    #[cfg(target_arch = "wasm32")]
    log::info!("Times: {:?}", times);
}

async fn execute_gpu(numbers: Vec<u32>) -> Vec<u32> {
    let slice_size = numbers.len() * std::mem::size_of::<u32>();
    let size = slice_size as wgpu::BufferAddress;

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: None,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        )
        .await
        .unwrap();

    let pipeline_statistics_queries = vec![wgt::PipelineStatisticName::ComputeShaderInvocations];

    let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
        type_: wgpu::QueryType::PipelineStatistics(&pipeline_statistics_queries),
        count: 1,
    });

    let cs_module = device.create_shader_module(wgpu::include_spirv!("shader.comp.spv"));

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    let query_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    });

    let storage_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(&numbers),
        wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
    );

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry::new(
            0,
            wgpu::ShaderStage::COMPUTE,
            wgpu::BindingType::StorageBuffer {
                dynamic: false,
                readonly: false,
                min_binding_size: wgpu::BufferSize::new(4),
            },
        )],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(storage_buffer.slice(..)),
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        layout: &pipeline_layout,
        compute_stage: wgpu::ProgrammableStageDescriptor {
            module: &cs_module,
            entry_point: "main",
        },
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass();
        cpass.begin_pipeline_statistics_query(&query_set, 0);
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch(numbers.len() as u32, 1, 1);
        cpass.end_pipeline_statistics_query();
    }
    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);
    encoder.resolve_query_set(&query_set, 0, 1, &query_buffer, 0);

    queue.submit(Some(encoder.finish()));

    // Note that we're not calling `.await` here.
    let buffer_slice = staging_buffer.slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

    let query_buffer_slice = query_buffer.slice(..);
    let query_buffer_future = query_buffer_slice.map_async(wgpu::MapMode::Read);

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    device.poll(wgpu::Maintain::Wait);

    if let Ok(()) = query_buffer_future.await {
        let data = query_buffer_slice.get_mapped_range();
        let result: Vec<_> = data
            .chunks_exact(8)
            .map(|b| u64::from_ne_bytes(b.try_into().unwrap()))
            .collect();

        println!("raw query data: {:?}", result);

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        query_buffer.unmap();
    } else {
        panic!("failed to retrieve query information from gpu");
    }

    if let Ok(()) = buffer_future.await {
        let data = buffer_slice.get_mapped_range();
        let result = data
            .chunks_exact(4)
            .map(|b| u32::from_ne_bytes(b.try_into().unwrap()))
            .collect();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap();

        result
    } else {
        panic!("failed to run compute on gpu!")
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();

        #[cfg(feature = "subscriber")]
        {
            let chrome_tracing_dir = std::env::var("WGPU_CHROME_TRACING");
            wgpu::util::initialize_default_subscriber(chrome_tracing_dir.ok());
        };

        futures::executor::block_on(run());
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        wasm_bindgen_futures::spawn_local(run());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_1() {
        let input = vec![1, 2, 3, 4];
        futures::executor::block_on(assert_execute_gpu(input, vec![0, 1, 7, 2]));
    }

    #[test]
    fn test_compute_2() {
        let input = vec![5, 23, 10, 9];
        futures::executor::block_on(assert_execute_gpu(input, vec![5, 15, 6, 19]));
    }

    #[test]
    fn test_multithreaded_compute() {
        use std::{sync::mpsc, thread, time::Duration};

        let thread_count = 8;

        let (tx, rx) = mpsc::channel();
        for _ in 0..thread_count {
            let tx = tx.clone();
            thread::spawn(move || {
                let input = vec![100, 100, 100];
                futures::executor::block_on(assert_execute_gpu(input, vec![25, 25, 25]));
                tx.send(true).unwrap();
            });
        }

        for _ in 0..thread_count {
            rx.recv_timeout(Duration::from_secs(10))
                .expect("A thread never completed.");
        }
    }

    async fn assert_execute_gpu(input: Vec<u32>, expected: Vec<u32>) {
        assert_eq!(execute_gpu(input).await, expected);
    }
}
