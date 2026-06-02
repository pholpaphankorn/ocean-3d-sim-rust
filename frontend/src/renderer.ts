import { oceanShader, dolphinShader } from './shaders';

export class Renderer {
  private device: GPUDevice;
  private canvas: HTMLCanvasElement;
  private pipeline: GPURenderPipeline;
  private dolphinPipeline: GPURenderPipeline; // ← Add this field
  private depthTexture: GPUTexture;
  private uniformBuffer: GPUBuffer;
  private bindGroup: GPUBindGroup;
  private indexBuffer: GPUBuffer | null = null;
  private indexCount: number = 0;

  constructor(canvas: HTMLCanvasElement, device: GPUDevice, format: GPUTextureFormat) {
    this.canvas = canvas;
    this.device = device;

    // --- SHADERS ---
    const oceanModule = device.createShaderModule({ code: oceanShader });
    const dolphinModule = device.createShaderModule({ code: dolphinShader }); // ← Compile dolphin shader

    // --- UNIFORMS (Shared by both pipelines) ---
    this.uniformBuffer = device.createBuffer({
      size: 64, // 4x4 matrix
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    // --- BIND GROUP LAYOUT ---
    const bindGroupLayout = device.createBindGroupLayout({
      entries: [{ binding: 0, visibility: GPUShaderStage.VERTEX, buffer: { type: 'uniform' } }],
    });

    this.bindGroup = device.createBindGroup({
      layout: bindGroupLayout,
      entries: [{ binding: 0, resource: { buffer: this.uniformBuffer } }],
    });

    const pipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [bindGroupLayout] });

    // --- 1. OCEAN PIPELINE ---
    this.pipeline = device.createRenderPipeline({
      layout: pipelineLayout,
      vertex: {
        module: oceanModule,
        entryPoint: 'vs_main',
        buffers: [
          {
            arrayStride: 16, // 4 floats * 4 bytes
            attributes: [
              { shaderLocation: 0, offset: 0, format: 'float32x3' }, // xyz
              { shaderLocation: 1, offset: 12, format: 'float32' }, // color_y
            ],
          },
        ],
      },
      fragment: { module: oceanModule, entryPoint: 'fs_main', targets: [{ format }] },
      primitive: { topology: 'triangle-list', cullMode: 'none' },
      depthStencil: { depthWriteEnabled: true, depthCompare: 'less', format: 'depth24plus' },
    });

    // --- 2. DOLPHIN PIPELINE (New) ---
    this.dolphinPipeline = device.createRenderPipeline({
      layout: pipelineLayout,
      vertex: {
        module: dolphinModule,
        entryPoint: 'vs_main',
        buffers: [
          {
            arrayStride: 16, // 4 floats * 4 bytes (Matching our Rust generator stride)
            attributes: [
              { shaderLocation: 0, offset: 0, format: 'float32x3' }, // xyz
              { shaderLocation: 1, offset: 12, format: 'float32' }, // lighting/intensity flag
            ],
          },
        ],
      },
      fragment: { module: dolphinModule, entryPoint: 'fs_main', targets: [{ format }] },
      // We use triangle-strip here since it's highly efficient for procedural tube/capsule segments
      primitive: { topology: 'triangle-strip', cullMode: 'none' },
      depthStencil: { depthWriteEnabled: true, depthCompare: 'less', format: 'depth24plus' },
    });

    // --- DEPTH TEXTURE ---
    this.depthTexture = device.createTexture({
      size: [canvas.width, canvas.height],
      format: 'depth24plus',
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
  }

  // --- Keep your uploadIndices function here ---
  uploadIndices(indexData: Uint32Array): void {
    this.indexCount = indexData.length;
    this.indexBuffer = this.device.createBuffer({
      size: indexData.byteLength,
      usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
    });
    this.device.queue.writeBuffer(this.indexBuffer, 0, indexData);
  }

  // --- Keep your existing draw() function for the ocean completely intact ---
  draw(context: GPUCanvasContext, verts: Float32Array, mvp: Float32Array): void {
    if (!this.indexBuffer) throw new Error('Index buffer not uploaded');

    // Always rewrite MVP to the uniform buffer at the start of the frame
    this.device.queue.writeBuffer(this.uniformBuffer, 0, mvp);

    const vertBuffer = this.device.createBuffer({
      size: verts.byteLength,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    this.device.queue.writeBuffer(vertBuffer, 0, verts);

    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: context.getCurrentTexture().createView(),
          clearValue: { r: 0.05, g: 0.1, b: 0.2, a: 1 },
          loadOp: 'clear', // This pass CLEARS the screen first
          storeOp: 'store',
        },
      ],
      depthStencilAttachment: {
        view: this.depthTexture.createView(),
        depthClearValue: 1.0,
        depthLoadOp: 'clear',
        depthStoreOp: 'store',
      },
    });

    pass.setPipeline(this.pipeline);
    pass.setBindGroup(0, this.bindGroup);
    pass.setVertexBuffer(0, vertBuffer);
    pass.setIndexBuffer(this.indexBuffer, 'uint32');
    pass.drawIndexed(this.indexCount);
    pass.end();

    this.device.queue.submit([encoder.finish()]);
  }

  // ─── NEW DRAW METHOD FOR THE DOLPHIN ───────────────────────────────────────
  drawDolphin(context: GPUCanvasContext, verts: Float32Array): void {
    // 1. Create a transient GPU buffer containing the newly flexed dolphin positions
    const vertBuffer = this.device.createBuffer({
      size: verts.byteLength,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    this.device.queue.writeBuffer(vertBuffer, 0, verts);

    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: context.getCurrentTexture().createView(),
          loadOp: 'load', // CRITICAL: 'load' draws over the ocean instead of wiping it out!
          storeOp: 'store',
        },
      ],
      depthStencilAttachment: {
        view: this.depthTexture.createView(),
        depthLoadOp: 'load', // CRITICAL: Keeps ocean depths intact so parts underwater hide correctly!
        depthStoreOp: 'store',
      },
    });

    // Bind the dolphin setup
    pass.setPipeline(this.dolphinPipeline);
    pass.setBindGroup(0, this.bindGroup); // Reuses the exact same MVP matrices seamlessly!
    pass.setVertexBuffer(0, vertBuffer);

    // Draw via non-indexed sequence because it's a procedural continuous tube strip
    pass.draw(verts.length / 4);

    pass.end();
    this.device.queue.submit([encoder.finish()]);
  }
}
