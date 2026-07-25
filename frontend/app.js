// ===== Tauri API Access =====
// Tauri 2 with withGlobalTauri: true exposes API at window.__TAURI__
let invoke;

try {
  // Try Tauri 2 path
  if (window.__TAURI__?.core?.invoke) {
    invoke = window.__TAURI__.core.invoke;
  } else if (window.__TAURI__?.invoke) {
    invoke = window.__TAURI__.invoke;
  } else {
    throw new Error('Tauri API not found');
  }
} catch (e) {
  console.error('Failed to access Tauri API:', e);
  // Mock for testing in browser
  invoke = async (cmd, args) => {
    console.log('Mock invoke:', cmd, args);
    if (cmd === 'get_tool_statuses') {
      return [
        { id: 'codex', name: 'Codex CLI', config_exists: true, current_url: 'https://api.example.com/codex', format: 'toml', config_dir: '.codex', config_file: 'config.toml' },
        { id: 'claude', name: 'Claude Code', config_exists: true, current_url: 'https://api.example.com/claude', format: 'json', config_dir: '.claude', config_file: 'settings.json' },
        { id: 'gemini', name: 'Gemini CLI', config_exists: false, format: 'env', config_dir: '.gemini', config_file: '.env' }
      ];
    }
    if (cmd === 'get_user_config') {
      return {
        presetRoutes: [
          { id: 'global', name: '全球高保', urls: { codex: 'https://api.aicodemirror.ai/api/codex', claude: 'https://api.aicodemirror.ai/api/claudecode', gemini: 'https://api.aicodemirror.ai/api/gemini' } },
          { id: 'domestic', name: '国内优化', urls: { codex: 'https://api.claudecode.net.cn/api/codex', claude: 'https://api.claudecode.net.cn/api/claudecode', gemini: 'https://api.claudecode.net.cn/api/gemini' } }
        ],
        customRoutes: []
      };
    }
    if (cmd === 'check_env') {
      return { node_installed: true, npm_installed: true, node_version: 'v20.11.0', npm_version: '10.2.4' };
    }
    return { success: true };
  };
}

// ===== State =====
let userConfig = null;
let toolStatuses = [];
let envStatus = null;
let pendingSwitch = null;

// ===== Init =====
async function init() {
  try {
    showLoading();
    const [statuses, config, env] = await Promise.all([
      invoke('get_tool_statuses').catch(e => { console.error('get_tool_statuses failed:', e); return []; }),
      invoke('get_user_config').catch(e => { console.error('get_user_config failed:', e); return { presetRoutes: [], customRoutes: [] }; }),
      invoke('check_env').catch(e => { console.error('check_env failed:', e); return { node_installed: false, npm_installed: false }; })
    ]);
    toolStatuses = statuses || [];
    userConfig = config || { presetRoutes: [], customRoutes: [] };
    envStatus = env || { node_installed: false, npm_installed: false };
    renderCards();
    renderEnvStatus();
  } catch (e) {
    console.error('Init failed:', e);
    showError('初始化失败: ' + e.message);
  }
}

function showLoading() {
  document.getElementById('cards-grid').innerHTML = `
    <div class="col-span-full flex items-center justify-center py-20">
      <span class="material-symbols-outlined animate-spin text-2xl" style="color: var(--primary);">progress_activity</span>
      <span class="ml-3" style="color: var(--secondary);">加载中...</span>
    </div>
  `;
}

function showError(msg) {
  document.getElementById('cards-grid').innerHTML = `
    <div class="col-span-full flex flex-col items-center justify-center py-20 text-center">
      <span class="material-symbols-outlined text-4xl mb-4" style="color: var(--error);">error</span>
      <p class="text-lg font-medium mb-2">出错了</p>
      <p class="text-sm" style="color: var(--on-surface-variant);">${escape(msg)}</p>
      <button onclick="init()" class="btn-primary mt-4">重试</button>
    </div>
  `;
}

// ===== Render: Cards =====
function renderCards() {
  if (!toolStatuses.length) {
    showError('没有获取到工具状态');
    return;
  }

  const grid = document.getElementById('cards-grid');
  grid.innerHTML = toolStatuses.map(tool => {
    const statusBadge = tool.config_exists
      ? (tool.error
        ? `<span class="status-badge status-error">● 解析失败</span>`
        : `<span class="status-badge status-ok">● 正常</span>`)
      : `<span class="status-badge status-error">● 未检测到</span>`;

    const urlBlock = tool.config_exists
      ? (tool.error
        ? `<div class="url-box" style="background: var(--error-container); border-color: var(--error);"><p style="color: var(--error);">${escape(tool.error)}</p></div>`
        : `<div class="url-box code">${escape(tool.current_url || '(空)')}</div>`)
      : `<div class="url-box text-center" style="background: transparent; border-style: dashed;"><span style="color: var(--secondary);">等待安装后显示线路</span></div>`;

    const routeButtons = (userConfig.presetRoutes || []).map(route => {
      const url = route.urls?.[tool.id];
      const isActive = tool.current_url && url && tool.current_url === url;
      return `<button onclick="handleSwitch('${tool.id}', '${escape(url || '')}', '${escape(route.name)}')" class="w-full ${isActive ? 'btn-primary' : 'btn-outline'} flex items-center justify-center gap-1">${isActive ? '<span class="material-symbols-outlined text-sm">fiber_manual_record</span>' : ''}${escape(route.name)}</button>`;
    }).join('');

    // 只显示该工具的自定义线路
    const toolCustomRoutes = (userConfig.customRoutes || []).filter(r => r.toolId === tool.id);
    const customButtons = toolCustomRoutes.map((route, i) => {
      const isActive = tool.current_url && route.url && tool.current_url === route.url;
      return `<button onclick="handleSwitch('${tool.id}', '${escape(route.url)}', '${escape(route.name)}')" class="w-full ${isActive ? 'btn-primary' : 'btn-outline'} flex items-center justify-center gap-1">${isActive ? '<span class="material-symbols-outlined text-sm">fiber_manual_record</span>' : ''}${escape(route.name)}</button>`;
    }).join('');

    const addButton = `<button onclick="openAddRoute('${tool.id}')" class="w-full btn-ghost border border-dashed flex items-center justify-center gap-1" style="border-color: var(--outline);"><span class="material-symbols-outlined text-sm">add</span>添加自定义线路</button>`;

    // 构建配置操作按钮组
    // 统一使用 flex 布局，所有按钮等宽排列，无论按钮数量
    const hasAuthFile = tool.id === 'codex';
    const configButtonText = tool.config_file === '.env' ? '.env' : 
                             tool.config_file === 'settings.json' ? 'settings' : 
                             tool.config_file;
    const configButtons = tool.config_exists
      ? `<div class="pt-4 border-t" style="border-color: var(--outline-variant);">
           <div class="flex gap-2">
             <button onclick="openConfigDir('${tool.config_dir}')" class="flex-1 btn-ghost text-xs py-2">打开目录</button>
             <button onclick="openConfigFile('${tool.config_dir}', '${tool.config_file}')" class="flex-1 btn-ghost text-xs py-2">打开 ${configButtonText}</button>
             ${hasAuthFile ? `<button onclick="openAuthFile('${tool.config_dir}', '${tool.id}')" class="flex-1 btn-ghost text-xs py-2">打开 auth</button>` : ''}
           </div>
         </div>`
      : `<div class="pt-4 border-t space-y-2" style="border-color: var(--outline-variant);">${envStatus?.node_installed && envStatus?.npm_installed ? `<button onclick="handleInstall('${tool.id}')" class="w-full btn-primary">安装 ${escape(tool.name)}</button>` : `<button disabled class="w-full btn-ghost opacity-50 cursor-not-allowed">安装 ${escape(tool.name)} (需 Node.js)</button>`}<button onclick="openConfigDir('${tool.config_dir}')" class="w-full btn-ghost">打开目录</button></div>`;

    const iconMap = { codex: 'terminal', claude: 'smart_toy', gemini: 'stars' };
    const icon = iconMap[tool.id] || 'apps';

    return `<div class="card p-5 flex flex-col gap-4">
<div class="flex justify-between items-start">
  <div class="flex items-center gap-3">
    <div class="w-10 h-10 rounded-lg flex items-center justify-center" style="background: var(--surface-container);">
      <span class="material-symbols-outlined" style="color: var(--primary);">${icon}</span>
    </div>
    <h2 class="text-lg font-semibold">${escape(tool.name)}</h2>
  </div>
  ${statusBadge}
</div>
<div class="space-y-2">
  ${tool.config_exists && !tool.error ? `<p class="text-sm" style="color: var(--on-surface-variant);">当前线路: <span class="font-semibold" style="color: var(--primary);">${escape(getCurrentRouteName(tool))}</span></p>` : ''}
  ${urlBlock}
</div>
<div class="space-y-2 mt-2">
  ${routeButtons}
  ${customButtons}
  ${addButton}
</div>
${configButtons}
</div>`;
  }).join('');
}

function getCurrentRouteName(tool) {
  if (!tool.current_url) return '未知';
  // 检查预设线路
  for (const route of (userConfig.presetRoutes || [])) {
    if (route.urls && route.urls[tool.id] === tool.current_url) {
      return route.name;
    }
  }
  // 检查自定义线路 - 需要匹配当前工具的自定义线路
  for (const route of (userConfig.customRoutes || [])) {
    if (route.toolId === tool.id && route.url === tool.current_url) {
      return route.name;
    }
  }
  return '自定义';
}

// ===== Render: Env Status =====
function renderEnvStatus() {
  const nodeEl = document.getElementById('node-status');
  const npmEl = document.getElementById('npm-status');
  const readyEl = document.getElementById('env-ready');

  if (envStatus?.node_installed) {
    nodeEl.innerHTML = `<span class="material-symbols-outlined text-sm" style="color: var(--tertiary);">terminal</span><span style="color: var(--tertiary);">Node.js ${envStatus.node_version || ''}</span>`;
  } else {
    nodeEl.innerHTML = `<span class="material-symbols-outlined text-sm" style="color: var(--error);">terminal</span><span style="color: var(--error);">Node.js 未安装</span>`;
  }

  if (envStatus?.npm_installed) {
    npmEl.innerHTML = `<span class="material-symbols-outlined text-sm" style="color: var(--tertiary);">package</span><span style="color: var(--tertiary);">npm ${envStatus.npm_version || ''}</span>`;
  } else {
    npmEl.innerHTML = `<span class="material-symbols-outlined text-sm" style="color: var(--error);">package</span><span style="color: var(--error);">npm 未安装</span>`;
  }

  if (envStatus?.node_installed && envStatus?.npm_installed) {
    readyEl.innerHTML = `<span class="material-symbols-outlined text-sm" style="color: var(--tertiary);">check_circle</span><span class="font-semibold" style="color: var(--tertiary);">环境就绪</span>`;
  } else {
    readyEl.innerHTML = `<span class="material-symbols-outlined text-sm" style="color: var(--secondary);">warning</span><span style="color: var(--secondary);">切换可用，安装需 Node.js</span>`;
  }
}

// ===== Switch Route =====
function handleSwitch(toolId, targetUrl, routeName) {
  const tool = toolStatuses.find(t => t.id === toolId);
  if (!tool) return;

  if (!tool.config_exists) {
    showToast(`${tool.name} 配置文件不存在，可能未安装`, 'warning');
    return;
  }

  document.getElementById('switch-tool-name').textContent = tool.name;
  document.getElementById('switch-from').textContent = getCurrentRouteName(tool);
  document.getElementById('switch-to').textContent = routeName;
  document.getElementById('switch-current-url').textContent = tool.current_url || '(空)';
  document.getElementById('switch-target-url').textContent = targetUrl;

  pendingSwitch = { toolId, targetUrl, toolName: tool.name };
  openModal('modal-switch');
}

document.getElementById('btn-confirm-switch').addEventListener('click', async () => {
  if (!pendingSwitch) return;
  closeModal('modal-switch');

  const { toolId, targetUrl, toolName } = pendingSwitch;
  showToast(`正在切换 ${toolName}...`, 'info');

  try {
    const result = await invoke('switch_route', { toolId, targetUrl });
    if (result?.success) {
      showToast(`${toolName} 切换成功！`, 'success');
      await refresh();
    } else {
      showToast(`${toolName} 切换失败: ${result?.error || '未知错误'}`, 'error');
    }
  } catch (e) {
    showToast(`切换失败: ${e.message}`, 'error');
  }
  pendingSwitch = null;
});

// ===== Add Route =====
let pendingAddToolId = null;

function openAddRoute(toolId) {
  pendingAddToolId = toolId;
  const tool = toolStatuses.find(t => t.id === toolId);
  const toolName = tool ? tool.name : toolId;
  document.getElementById('add-route-tool-name').textContent = toolName;
  document.getElementById('routeName').value = '';
  document.getElementById('routeUrl').value = '';
  openModal('modal-add-route');
}

async function saveNewRoute() {
  const name = document.getElementById('routeName').value.trim();
  const url = document.getElementById('routeUrl').value.trim();

  if (!pendingAddToolId) {
    showToast('错误：未指定工具', 'error');
    return;
  }
  if (!name) { showToast('线路名称不能为空', 'warning'); return; }
  if (!url) { showToast('URL 不能为空', 'warning'); return; }
  if (!url.startsWith('http://') && !url.startsWith('https://')) {
    showToast('URL 需以 http:// 或 https:// 开头', 'warning'); return;
  }

  try {
    await invoke('add_custom_route', { toolId: pendingAddToolId, name, url });
    showToast('自定义线路添加成功！', 'success');
    closeModal('modal-add-route');
    pendingAddToolId = null;
    await refresh();
  } catch (e) {
    showToast(`添加失败: ${e.message}`, 'error');
  }
}

// ===== Manage Routes =====
async function openManageRoutes() {
  await refresh();
  renderManageRoutes();
  openModal('modal-manage-routes');
}

function renderManageRoutes() {
  const list = document.getElementById('custom-routes-list');
  const routes = userConfig?.customRoutes || [];

  if (routes.length === 0) {
    list.innerHTML = '<div class="p-8 text-center" style="color: var(--secondary);">暂无自定义线路</div>';
    document.getElementById('route-count').textContent = '共 0 条自定义线路';
    return;
  }

  // 按工具分组
  const routesByTool = {};
  toolStatuses.forEach(t => routesByTool[t.id] = []);
  routes.forEach((route, i) => {
    if (!routesByTool[route.toolId]) routesByTool[route.toolId] = [];
    routesByTool[route.toolId].push({ ...route, index: i });
  });

  let html = '';
  toolStatuses.forEach(tool => {
    const toolRoutes = routesByTool[tool.id] || [];
    if (toolRoutes.length === 0) return;

    html += `<div class="p-3 font-semibold border-b" style="background: var(--surface-container); border-color: var(--outline-variant);">${escape(tool.name)}</div>`;
    html += toolRoutes.map((route, i) => `
      <div class="p-4 flex items-center justify-between ${i < toolRoutes.length - 1 ? 'border-b' : ''}" style="border-color: var(--outline-variant);">
        <div>
          <p class="font-medium">${escape(route.name)}</p>
          <p class="code text-sm" style="color: var(--secondary);">${escape(route.url)}</p>
        </div>
        <div class="flex gap-2">
          <button onclick="editRoute(${route.index})" class="px-3 py-1 text-sm border rounded" style="border-color: var(--outline-variant);">编辑</button>
          <button onclick="deleteRoute(${route.index})" class="px-3 py-1 text-sm border rounded" style="border-color: var(--error); color: var(--error);">删除</button>
        </div>
      </div>
    `).join('');
  });

  list.innerHTML = html || '<div class="p-8 text-center" style="color: var(--secondary);">暂无自定义线路</div>';
  document.getElementById('route-count').textContent = `共 ${routes.length} 条自定义线路`;
}

async function deleteRoute(index) {
  const route = (userConfig?.customRoutes || [])[index];
  if (!route) return;
  if (!confirm(`确认删除「${route.name}」？`)) return;

  try {
    await invoke('delete_custom_route', { index });
    showToast('自定义线路已删除', 'success');
    await refresh();
    renderManageRoutes();
  } catch (e) {
    showToast(`删除失败: ${e.message}`, 'error');
  }
}

async function editRoute(index) {
  const route = (userConfig?.customRoutes || [])[index];
  if (!route) return;
  closeModal('modal-manage-routes');
  document.getElementById('routeName').value = route.name;
  document.getElementById('routeUrl').value = route.url;
  openModal('modal-add-route');

  const saveBtn = document.querySelector('button[onclick="saveNewRoute()"]');
  saveBtn.onclick = async () => {
    const name = document.getElementById('routeName').value.trim();
    const url = document.getElementById('routeUrl').value.trim();
    if (!name) { showToast('线路名称不能为空', 'warning'); return; }
    if (!url || (!url.startsWith('http://') && !url.startsWith('https://'))) {
      showToast('URL 格式无效', 'warning'); return;
    }
    try {
      await invoke('edit_custom_route', { index, name, url });
      showToast('自定义线路更新成功！', 'success');
      closeModal('modal-add-route');
      await refresh();
      saveBtn.onclick = saveNewRoute;
    } catch (e) {
      showToast(`更新失败: ${e.message}`, 'error');
    }
  };
}

// ===== Open Config =====
async function openConfigDir(configDir) {
  try {
    await invoke('open_config_dir', { configDir });
    showToast('已打开配置目录', 'success');
  } catch (e) {
    showToast(`打开失败: ${e.message}`, 'error');
  }
}

async function openConfigFile(configDir, configFile) {
  try {
    await invoke('open_config_file', { configDir, configFile });
    showToast('已打开配置文件', 'success');
  } catch (e) {
    showToast(`打开失败: ${e.message}`, 'error');
  }
}

async function openAuthFile(configDir, toolId) {
  // Codex 的 auth 文件是 auth.json
  const authFile = 'auth.json';

  try {
    await invoke('open_config_file', { configDir, configFile: authFile });
    showToast('已打开 auth 文件', 'success');
  } catch (e) {
    showToast(`打开失败: ${e.message}`, 'error');
  }
}

// ===== Install Tool =====
let pendingInstall = null;

async function handleInstall(toolId) {
  const tool = toolStatuses.find(t => t.id === toolId);
  if (!tool) return;

  const routeOptions = [
    ...(userConfig?.presetRoutes || []).map(r => ({ name: r.name, url: r.urls?.[toolId] })),
    ...(userConfig?.customRoutes || []).filter(r => r.toolId === toolId).map(r => ({ name: r.name, url: r.url })),
  ].filter(r => r.url);

  if (routeOptions.length === 0) {
    showToast('没有可用线路', 'warning');
    return;
  }

  pendingInstall = { toolId, toolName: tool.name, routeOptions };

  document.getElementById('install-tool-name').textContent = tool.name;
  const list = document.getElementById('install-route-list');
  list.innerHTML = routeOptions.map((route, i) => `
    <button onclick="confirmInstall(${i})" class="w-full p-3 border rounded-lg flex items-center justify-between hover:bg-gray-50" style="border-color: var(--outline-variant);">
      <span class="font-medium">${escape(route.name)}</span>
      <span class="code text-xs" style="color: var(--secondary);">${escape(route.url)}</span>
    </button>
  `).join('');

  openModal('modal-install-route');
}

async function confirmInstall(idx) {
  if (!pendingInstall) return;
  const { toolId, toolName, routeOptions } = pendingInstall;
  if (isNaN(idx) || idx < 0 || idx >= routeOptions.length) {
    showToast('已取消安装', 'warning');
    closeModal('modal-install-route');
    pendingInstall = null;
    return;
  }

  const targetUrl = routeOptions[idx].url;
  closeModal('modal-install-route');
  showToast(`正在安装 ${toolName}...`, 'info');

  try {
    const result = await invoke('install_tool', { toolId, targetUrl });
    if (result?.success) {
      showToast(`${toolName} 安装并配置成功！`, 'success');
      await refresh();
    } else {
      showToast(`${toolName} 安装失败: ${result?.error || '未知错误'}`, 'error');
    }
  } catch (e) {
    showToast(`安装失败: ${e.message}`, 'error');
  }
  pendingInstall = null;
}

// ===== Utils =====
async function refresh() {
  try {
    const [statuses, config] = await Promise.all([
      invoke('get_tool_statuses').catch(() => []),
      invoke('get_user_config').catch(() => ({ presetRoutes: [], customRoutes: [] }))
    ]);
    toolStatuses = statuses || [];
    userConfig = config || { presetRoutes: [], customRoutes: [] };
    renderCards();
  } catch (e) {
    console.error('Refresh failed:', e);
  }
}

function openModal(id) { document.getElementById(id).classList.remove('hidden'); }
function closeModal(id) { document.getElementById(id).classList.add('hidden'); }

function showToast(msg, type = 'info') {
  const container = document.getElementById('toast-container');
  const colors = {
    success: 'var(--tertiary)',
    error: 'var(--error)',
    warning: '#eab308',
    info: 'var(--secondary)'
  };
  const toast = document.createElement('div');
  toast.className = 'toast';
  toast.style.background = colors[type] || colors.info;
  toast.textContent = msg;
  container.appendChild(toast);
  setTimeout(() => toast.remove(), 3000);
}

function escape(s) {
  if (!s) return '';
  return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}

// 从管理弹窗打开添加线路 — 弹出工具选择
function openAddRouteFromManage() {
  const tools = toolStatuses.filter(t => t.config_exists !== undefined);
  if (tools.length === 0) {
    showToast('请先加载工具状态', 'warning');
    return;
  }
  if (tools.length === 1) {
    openAddRoute(tools[0].id);
    return;
  }
  // 多个工具时弹出选择列表
  const list = tools.map((t, i) => `${i + 1}. ${t.name}`).join('\n');
  const choice = prompt(`选择要添加线路的工具:\n${list}`);
  const idx = parseInt(choice) - 1;
  if (isNaN(idx) || idx < 0 || idx >= tools.length) {
    showToast('已取消', 'warning');
    return;
  }
  openAddRoute(tools[idx].id);
}

// ===== Start =====
document.addEventListener('DOMContentLoaded', init);
