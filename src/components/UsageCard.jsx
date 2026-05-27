import React, { useState, useEffect, useRef } from 'react'
import { Pin, Settings, LogOut, Github, X } from 'lucide-react'
import { open } from '@tauri-apps/plugin-shell'
import { invoke } from '@tauri-apps/api/core'

const GITEE_URL = 'https://gitee.com/genmers/zhipu-usage-api'
const GITHUB_URL = 'https://github.com/Genmer/zhipu-usage-api'
const GITHUB_API = 'https://api.github.com/repos/Genmer/zhipu-usage-api/releases/latest'

const RADIUS = 18
const CIRCUMFERENCE = 2 * Math.PI * RADIUS

const CircularProgress = ({ percentage }) => {
  const numVal = parseInt(percentage) || 0
  const color = numVal > 80 ? '#ef4444' : numVal > 60 ? '#f59e0b' : '#3b82f6'
  const offset = CIRCUMFERENCE * (1 - numVal / 100)

  return (
    <svg width="48" height="48" viewBox="0 0 48 48">
      <circle cx="24" cy="24" r={RADIUS} fill="none" stroke="rgba(255,255,255,0.1)" strokeWidth="4" />
      <circle cx="24" cy="24" r={RADIUS} fill="none" stroke={color} strokeWidth="4"
        strokeDasharray={CIRCUMFERENCE} strokeDashoffset={offset} strokeLinecap="round"
        transform="rotate(-90 24 24)"
        style={{ transition: 'stroke-dashoffset 0.7s ease-out, stroke 0.3s' }} />
      <text x="24" y="24.5" textAnchor="middle" dominantBaseline="central"
        fill="white" fontSize="12" fontWeight="bold">{numVal}%</text>
    </svg>
  )
}

const UsageCard = ({ title, percentage, resetTime, isHighUsage = false, otherPercentage, otherTitle, onNext, onPrev, isPinned, onTogglePin, isRefreshing, isSuccess, onRefresh, onOpenSettings, onLogout }) => {
  const numVal = parseInt(percentage) || 0
  const barColor = numVal > 80 ? '#ef4444' : numVal > 60 ? '#f59e0b' : '#3b82f6'
  const [hoverLogout, setHoverLogout] = useState(false)
  const [showCloseMenu, setShowCloseMenu] = useState(false)
  const [hoverGithub, setHoverGithub] = useState(false)
  const [latestVersion, setLatestVersion] = useState(null)
  const githubTimer = useRef(null)

  useEffect(() => {
    fetch(GITHUB_API)
      .then(r => r.json())
      .then(d => setLatestVersion(d.tag_name?.replace(/^v/, '') || null))
      .catch(() => {})
  }, [])

  const handleBackgroundRun = () => {
    setShowCloseMenu(false)
    // Dock 图标已通过 ActivationPolicy::Accessory 隐藏，悬浮窗正常显示
  }

  const handleQuit = () => {
    setShowCloseMenu(false)
    invoke('quit_app')
  }

  return (
    <div className="drag-region w-64 h-52 rounded-2xl overflow-hidden shadow-2xl flex flex-col"
         style={{ backgroundColor: 'rgba(30, 30, 30, 0.75)' }}>
      <div className="flex items-center justify-center pt-4 pb-1 relative">
        <button onClick={onTogglePin} className="no-drag absolute left-3 top-3 w-6 h-6 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.06)' }}>
          <Pin size={14} style={{ color: isPinned ? '#3b82f6' : '#6b7280', transform: isPinned ? 'rotate(45deg)' : 'rotate(0deg)', transition: 'all 0.2s' }} fill={isPinned ? '#3b82f6' : 'none'} />
        </button>
        <div className="no-drag absolute left-3 top-[36px]"
          onMouseEnter={() => { clearTimeout(githubTimer.current); setHoverGithub(true) }}
          onMouseLeave={() => { githubTimer.current = setTimeout(() => setHoverGithub(false), 200) }}>
          <button className="w-6 h-6 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: hoverGithub ? 'rgba(255,255,255,0.1)' : 'rgba(255,255,255,0.06)' }}>
            <Github size={12} className="text-gray-400" />
          </button>
          {hoverGithub && (
            <div className="absolute left-0 top-7 bg-gray-800/95 rounded-lg shadow-lg py-1.5 z-50 border border-white/5" style={{ minWidth: '160px' }}
              onMouseEnter={() => { clearTimeout(githubTimer.current); setHoverGithub(true) }}
              onMouseLeave={() => { githubTimer.current = setTimeout(() => setHoverGithub(false), 200) }}>
              <button onClick={() => open(GITHUB_URL)} className="no-drag w-full text-[10px] text-gray-300 hover:bg-white/10 px-3 py-1.5 text-left whitespace-nowrap transition-colors no-select flex items-center gap-2">
                <Github size={10} /> GitHub
                <span className="text-[8px] text-blue-400 ml-auto">安装包</span>
              </button>
              <button onClick={() => open(GITEE_URL)} className="no-drag w-full text-[10px] text-gray-300 hover:bg-white/10 px-3 py-1.5 text-left whitespace-nowrap transition-colors no-select flex items-center gap-2">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><path d="M11.984 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.016 0zm6.09 5.333c.328 0 .593.266.593.593v1.687a3.953 3.953 0 0 1 1.856 1.376l1.19-.685a.593.593 0 1 1 .592 1.027l-1.19.685a3.959 3.959 0 0 1 0 2.752l1.19.685a.593.593 0 1 1-.592 1.027l-1.19-.685a3.953 3.953 0 0 1-1.856 1.376v1.687a.593.593 0 1 1-1.186 0v-1.687a3.953 3.953 0 0 1-1.856-1.376l-1.19.685a.593.593 0 1 1-.592-1.027l1.19-.685a3.959 3.959 0 0 1 0-2.752l-1.19-.685a.593.593 0 1 1 .592-1.027l1.19.685a3.953 3.953 0 0 1 1.856-1.376V5.926c0-.327.266-.593.593-.593z"/></svg>
                Gitee
              </button>
              <div className="border-t border-white/5 mt-1 pt-1 px-3 pb-1">
                <div className="text-[9px] text-gray-500">当前 v{__APP_VERSION__}</div>
                {latestVersion && (
                  <div className="text-[9px] mt-0.5">
                    {latestVersion !== __APP_VERSION__ ? (
                      <span className="text-yellow-400">最新 v{latestVersion} ↑</span>
                    ) : (
                      <span className="text-green-400">已是最新</span>
                    )}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
        <span className="text-white text-xs font-bold tracking-wide">{title}</span>
        <button onClick={onRefresh} disabled={isRefreshing} className="no-drag absolute right-3 top-3 w-6 h-6 rounded-full flex items-center justify-center transition-all duration-300 no-select disabled:opacity-40 hover:bg-white/10" style={{ backgroundColor: isSuccess ? 'rgba(59, 130, 246, 0.3)' : 'rgba(255,255,255,0.06)' }}>
          {isRefreshing ? (
            <div className="w-3 h-3 border-[1.5px] border-gray-500 border-t-blue-400 rounded-full animate-spin" />
          ) : isSuccess ? (
            <svg className="w-3 h-3 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
            </svg>
          ) : (
            <svg className="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          )}
        </button>
        <button onClick={onOpenSettings} className="no-drag absolute right-3 top-9 w-6 h-6 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.06)' }}>
          <Settings size={12} className="text-gray-400" />
        </button>
        <div className="no-drag absolute right-3 top-[60px]">
          <button onClick={onLogout}
                  onMouseEnter={() => setHoverLogout(true)}
                  onMouseLeave={() => setHoverLogout(false)}
                  className="w-6 h-6 rounded-full flex items-center justify-center hover:bg-red-500/20 transition-colors no-select relative"
                  style={{ backgroundColor: hoverLogout ? 'rgba(239,68,68,0.2)' : 'rgba(255,255,255,0.06)' }}>
            <LogOut size={11} style={{ color: hoverLogout ? '#ef4444' : '#9ca3af' }} />
          </button>
          {hoverLogout && (
            <span className="absolute right-8 top-0.5 text-[9px] text-red-400 whitespace-nowrap no-select pointer-events-none">注销</span>
          )}
        </div>
      </div>

      <div className="flex-1 flex flex-col items-center justify-center -mt-2">
        <span className="text-white text-5xl font-bold tracking-tight">{percentage}</span>
      </div>
      {otherPercentage && (
        <div className="absolute no-drag cursor-default"
          style={{ right: '18px', top: '100px' }}>
          <CircularProgress percentage={otherPercentage} />
        </div>
      )}

      <div className="px-5 mb-3">
        <div className="w-full h-1.5 rounded-full" style={{ backgroundColor: 'rgba(255,255,255,0.1)' }}>
          <div className="h-full rounded-full transition-all duration-700 ease-out"
            style={{ width: `${numVal}%`, backgroundColor: barColor }} />
        </div>
      </div>

      <div className="flex items-center justify-between px-5 pb-3 relative">
        <span className="text-white text-[10px] font-bold">重置: {resetTime}</span>
        <div className="flex gap-1.5 items-center">
          <button onClick={onPrev} className="no-drag w-5 h-5 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.05)' }}>
            <svg className="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M15 19l-7-7 7-7" /></svg>
          </button>
          <button onClick={onNext} className="no-drag w-5 h-5 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.05)' }}>
            <svg className="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M9 5l7 7-7 7" /></svg>
          </button>
          <button onClick={() => setShowCloseMenu(!showCloseMenu)} className="no-drag w-5 h-5 rounded-full flex items-center justify-center hover:bg-red-500/20 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.05)' }}>
            <X size={10} className="text-gray-400" />
          </button>
        </div>
        {showCloseMenu && (
          <div className="absolute bottom-8 right-0 bg-gray-800/95 rounded-lg shadow-lg py-1 flex flex-col z-50 border border-white/5" style={{ minWidth: '72px' }}>
            <button onClick={handleBackgroundRun} className="no-drag text-[10px] text-gray-300 hover:bg-white/10 px-2.5 py-1.5 rounded-t-lg text-left whitespace-nowrap transition-colors no-select">
              后台运行
            </button>
            <button onClick={handleQuit} className="no-drag text-[10px] text-red-400 hover:bg-red-500/10 px-2.5 py-1.5 rounded-b-lg text-left whitespace-nowrap transition-colors no-select">
              完全退出
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

export default UsageCard
