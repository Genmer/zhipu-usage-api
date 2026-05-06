import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Trash2, Eye, EyeOff, Github } from 'lucide-react'
import { open } from '@tauri-apps/plugin-shell'

const LoginScreen = () => {
  const [accounts, setAccounts] = useState([])
  const [apiKey, setApiKey] = useState('')
  const [showKey, setShowKey] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [showInput, setShowInput] = useState(false)

  useEffect(() => {
    invoke('get_saved_accounts').then(setAccounts).catch(() => {})
  }, [])

  const handleRemove = async (id, e) => {
    e.stopPropagation()
    e.preventDefault()
    try {
      await invoke('remove_saved_account', { id })
      setAccounts(prev => prev.filter(a => a.id !== id))
    } catch (err) {
      console.error('remove account failed:', err)
    }
  }

  const handleAccountClick = async (id) => {
    setLoading(true)
    setError('')
    try {
      await invoke('login_with_saved_account', { id })
    } catch (err) {
      setError(err)
    } finally {
      setLoading(false)
    }
  }

  const handleSubmit = async () => {
    if (!apiKey.trim()) return
    setLoading(true)
    setError('')
    try {
      await invoke('login_with_api_key', { apiKey: apiKey.trim() })
    } catch (err) {
      setError(err)
    } finally {
      setLoading(false)
    }
  }

  const handleKeyDown = (e) => {
    if (e.key === 'Enter') handleSubmit()
  }

  return (
    <div className="drag-region w-64 h-52 rounded-2xl overflow-hidden shadow-2xl flex flex-col"
         style={{ backgroundColor: 'rgba(30, 30, 30, 0.75)' }}>
      <div className="flex items-center justify-center pt-4 pb-1">
        <span className="text-white text-xs font-bold tracking-wide">智谱AI 额度监控</span>
      </div>

      {!showInput ? (
        <div className="flex-1 flex flex-col px-3 overflow-hidden">
          {accounts.length > 0 && (
            <>
              <span className="text-gray-300 text-[9px] mb-1 px-1 font-medium">已记住的Key</span>
              <div className="flex-1 overflow-y-auto space-y-1 mb-2" style={{ scrollbarWidth: 'none' }}>
                {accounts.map(acc => (
                  <div key={acc.id}
                       onClick={() => handleAccountClick(acc.id)}
                       className="no-drag group flex items-center justify-between px-2 py-1.5 rounded-lg cursor-pointer transition-all"
                       style={{ backgroundColor: 'rgba(59, 130, 246, 0.15)', border: '1px solid rgba(59, 130, 246, 0.3)' }}>
                    <div className="flex-1 min-w-0">
                      <div className="text-white text-[11px] truncate font-medium font-mono">{acc.label}</div>
                      <div className="text-gray-300 text-[8px]">{acc.last_used}</div>
                    </div>
                    <button onClick={(e) => handleRemove(acc.id, e)}
                            className="no-drag w-5 h-5 flex items-center justify-center rounded transition-all"
                            style={{ backgroundColor: 'rgba(255,255,255,0.1)' }}>
                      <Trash2 size={11} className="text-gray-300 group-hover:text-red-400 transition-colors" />
                    </button>
                  </div>
                ))}
              </div>
            </>
          )}
          <div className="pb-2 space-y-1.5">
            <button onClick={() => { setShowInput(true); setShowKey(false); setError(''); }}
                    className="no-drag w-full py-1.5 rounded-lg text-[11px] font-medium text-gray-200 transition-all hover:bg-white/10 hover:text-white"
                    style={{ border: '1px solid rgba(255,255,255,0.2)' }}>
              + 输入新的 API Key
            </button>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex flex-col px-4 pt-1">
          <div className="flex items-center gap-1.5 mb-2">
            <button onClick={() => { setShowInput(false); setError(''); }}
                    className="no-drag text-gray-400 hover:text-white text-[10px] transition-colors">
              ← 返回
            </button>
            <span className="text-gray-500 text-[9px]">|</span>
            <span className="text-gray-300 text-[9px]">输入智谱API Key</span>
          </div>

          <div className="relative mb-2">
            <input
              type={showKey ? 'text' : 'password'}
              value={apiKey}
              onChange={e => setApiKey(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="输入 API Key..."
              disabled={loading}
              className="no-drag w-full h-7 rounded-lg px-2 pr-8 text-white text-[11px] font-mono outline-none border border-white/10 focus:border-blue-500 transition-colors"
              style={{ backgroundColor: 'rgba(255,255,255,0.08)' }}
            />
            <button onClick={() => setShowKey(!showKey)}
                    className="no-drag absolute right-1.5 top-1/2 -translate-y-1/2 w-5 h-5 flex items-center justify-center rounded hover:bg-white/10 transition-colors">
              {showKey ? <EyeOff size={11} className="text-gray-400" /> : <Eye size={11} className="text-gray-400" />}
            </button>
          </div>

          {error && (
            <p className="text-red-400 text-[9px] mb-1 px-0.5">{error}</p>
          )}

          <button onClick={handleSubmit}
                  disabled={loading || !apiKey.trim()}
                  className="no-drag w-full py-1.5 rounded-lg text-[11px] font-medium text-white transition-all hover:opacity-90 active:scale-95 disabled:opacity-40"
                  style={{ backgroundColor: '#3b82f6' }}>
            {loading ? (
              <div className="flex items-center justify-center gap-1.5">
                <div className="w-2.5 h-2.5 border-[1.5px] border-white/30 border-t-white rounded-full animate-spin" />
                <span>验证中...</span>
              </div>
            ) : '确认登录'}
          </button>

          <p className="text-gray-500 text-[8px] text-center mt-1.5">
            从 open.bigmodel.cn 获取 API Key
          </p>
        </div>
      )}

      <div className="flex items-center justify-center pb-2 gap-1.5">
        <span className="text-gray-300 text-[9px]">v3.0.0</span>
        <span className="text-gray-500 text-[9px]">·</span>
        <button onClick={() => open('https://gitee.com/genmers/zhipu-usage-api')}
                className="no-drag text-gray-400 hover:text-white transition-colors">
          <Github size={12} />
        </button>
      </div>
    </div>
  )
}

export default LoginScreen
