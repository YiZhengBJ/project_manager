/* Browser preview uses localStorage. The desktop app uses Rust-managed JSON. */
window.projectStorage = (() => {
  const native = Boolean(window.__TAURI__?.core);
  const invoke = (command, args) => window.__TAURI__.core.invoke(command, args);
  let pending = Promise.resolve();
  let lastError = null;
  return {
    native,
    async load() {
      if (native) return invoke('load_project');
      const value = localStorage.getItem('branch-v1');
      return value ? JSON.parse(value) : null;
    },
    save(workspace) {
      const snapshot = structuredClone(workspace);
      if (!native) {
        try { localStorage.setItem('branch-v1', JSON.stringify(snapshot)); return Promise.resolve(); }
        catch (error) { return Promise.reject(error); }
      }
      const operation = pending.then(() => invoke('save_project', {workspace: snapshot}));
      pending = operation.then(() => { lastError = null; }, error => { lastError = error; });
      return operation;
    },
    async flush() { await pending; if (lastError) throw lastError; },
    async exportExcel(bytes, filename) {
      if (native) return invoke('export_excel', {bytes: Array.from(bytes), filename});
      const blob = new Blob([bytes], {type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'});
      const url = URL.createObjectURL(blob), link = document.createElement('a');
      link.href=url; link.download=filename; document.body.append(link); link.click(); link.remove();
      setTimeout(()=>URL.revokeObjectURL(url), 30000);
      return '';
    },
    async copy(text) {
      if (native) return invoke('copy_text', {text});
      return navigator.clipboard.writeText(text);
    }
  };
})();

async function setupCloseHandler() {
  if (!window.projectStorage.native) return;
  const appWindow = window.__TAURI__.window.getCurrentWindow();
  await appWindow.onCloseRequested(async event => {
    event.preventDefault();
    try {
      await window.projectStorage.flush();
      await appWindow.destroy();
    } catch (error) {
      const toast = document.querySelector('#toast');
      toast.textContent = '保存失败，应用未关闭。请稍后重试：' + String(error);
      toast.classList.add('show');
    }
  });
}
