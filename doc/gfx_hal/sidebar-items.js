initSidebarItems({"enum":[["IndexType","An enum describing the type of an index value in a slice's index buffer"]],"macro":[["spec_const_list","Macro for specifying list of specialization constatns for `EntryPoint`."]],"mod":[["adapter","Physical devices and adapters."],["buffer","Memory buffers."],["command","Command buffers."],["device","Logical device"],["format","Universal format specification. Applicable to textures, views, and vertex buffers."],["image","Image related structures."],["memory","Types to describe the properties of memory allocated for gfx resources."],["pass","RenderPass handling."],["pool","Command pools"],["prelude","Prelude module re-exports all the traits necessary to use gfx-hal."],["pso","Raw Pipeline State Objects"],["query","Queries are commands that can be submitted to a command buffer to record statistics or other useful values as the command buffer is running. They are often intended for profiling or other introspection, providing a mechanism for the command buffer to record data about its operation as it is running."],["queue","Command queues."],["window","Windowing system interoperability"]],"struct":[["Features","Features that the device supports. These only include features of the core interface and not API extensions."],["Hints","Features that the device supports natively, but is able to emulate."],["Limits","Resource limits of a particular graphics device."],["MemoryTypeId","A strongly-typed index to a particular `MemoryType`."],["UnsupportedBackend","Error creating an instance of a backend on the platform that doesn't support this backend."]],"trait":[["Backend","The `Backend` trait wraps together all the types needed for a graphics backend. Each backend module, such as OpenGL or Metal, will implement this trait with its own concrete types."],["Instance","An instantiated backend."]],"type":[["DrawCount","Indirect draw calls count."],["IndexCount","Draw number of indices."],["InstanceCount","Draw number of instances."],["VertexCount","Draw vertex count."],["VertexOffset","Draw vertex base offset."],["WorkGroupCount","Number of work groups."]]});