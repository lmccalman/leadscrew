local lspconfig = require('lspconfig')

lspconfig.rust_analyzer.setup {
  settings = {
    ['rust-analyzer'] = {
      checkOnSave = {
        allTargets = false,
        -- Specify your target here
        extraArgs = {'--target', 'thumbv7em-none-eabihf'}
      },
      cargo = {
        -- Specify your target here as well
        target = 'thumbv7em-none-eabihf',
        -- If you're using a custom toolchain, specify it here
        -- toolchain = 'nightly-2023-01-01'
      },
      -- Other rust-analyzer settings can go here
    }
  }
}
