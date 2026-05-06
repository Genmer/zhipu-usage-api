import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Trash2 } from 'lucide-react'

const LoginScreen = ({ onLogin }) => {
  const [accounts, setAccounts] = useState([])

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
    try {
      await invoke('login_with_saved_account', { id })
    } catch (err) {
      console.error('login with saved account failed:', err)
    }
  }

  return (
    <div className="drag-region w-64 h-52 rounded-2xl overflow-hidden shadow-2xl flex flex-col"
         style={{ backgroundColor: 'rgba(30, 30, 30, 0.75)' }}>
      <div className="flex items-center justify-center pt-4 pb-1">
        <span className="text-white text-xs font-bold tracking-wide">智谱AI 额度监控</span>
      </div>

      {accounts.length > 0 ? (
        <div className="flex-1 flex flex-col px-3 overflow-hidden">
          <span className="text-gray-300 text-[9px] mb-1 px-1 font-medium">已记住的账号</span>
          <div className="flex-1 overflow-y-auto space-y-1 mb-2" style={{ scrollbarWidth: 'none' }}>
            {accounts.map(acc => (
              <div key={acc.id}
                   onClick={() => handleAccountClick(acc.id)}
                   className="no-drag group flex items-center justify-between px-2 py-1.5 rounded-lg cursor-pointer transition-all"
                   style={{ backgroundColor: 'rgba(59, 130, 246, 0.15)', border: '1px solid rgba(59, 130, 246, 0.3)' }}>
                <div className="flex-1 min-w-0">
                  <div className="text-white text-[11px] truncate font-medium">{acc.label}</div>
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
          <div className="pb-2">
            <button onClick={onLogin}
                    className="no-drag w-full py-1.5 rounded-lg text-[11px] font-medium text-gray-200 transition-all hover:bg-white/10 hover:text-white"
                    style={{ border: '1px solid rgba(255,255,255,0.2)' }}>
              + 使用其他账号登录
            </button>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex flex-col items-center justify-center px-6">
          <div className="w-12 h-12 rounded-2xl flex items-center justify-center mb-3"
               style={{ backgroundColor: 'rgba(59, 130, 246, 0.15)' }}>
            <svg className="w-6 h-6 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          </div>
          <p className="text-gray-200 text-[11px] text-center mb-0.5">登录后查看额度信息</p>
          <p className="text-gray-300 text-[9px] text-center mb-3">支持微信扫码或账号密码</p>
          <button onClick={onLogin}
                  className="no-drag w-full py-1.5 px-4 rounded-lg text-xs font-medium text-white transition-all hover:opacity-90 active:scale-95"
                  style={{ backgroundColor: '#3b82f6' }}>
            登录智谱AI
          </button>
        </div>
      )}

      <div className="flex items-center justify-center pb-2 gap-1.5">
        <span className="text-gray-300 text-[9px]">v2.0.0</span>
        <span className="text-gray-500 text-[9px]">·</span>
        <a href="https://gitee.com" target="_blank" rel="noopener noreferrer"
           className="no-drag text-gray-400 hover:text-white transition-colors"
           onClick={e => e.stopPropagation()}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/>
          </svg>
        </a>
      </div>
    </div>
  )
}

export default LoginScreen
