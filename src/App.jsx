import React, { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import LoginScreen from './components/LoginScreen'
import UsageCard from './components/UsageCard'
import SettingsDialog from './components/SettingsDialog'

function App() {
  const [isLoggedIn, setIsLoggedIn] = useState(false)
  const [currentCard, setCurrentCard] = useState(0)
  const [isAnimating, setIsAnimating] = useState(false)
  const [isPinned, setIsPinned] = useState(true)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [isSuccess, setIsSuccess] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [cardSwitchSeconds, setCardSwitchSeconds] = useState(30)
  const [usageData, setUsageData] = useState([
    { title: "每5小时额度", percentage: "0%", resetTime: "加载中...", isHighUsage: false },
    { title: "每周额度", percentage: "0%", resetTime: "加载中...", isHighUsage: false }
  ])

  const handleDragStart = useCallback((e) => {
    const tag = e.target.tagName.toLowerCase()
    if (tag === 'button' || tag === 'input' || tag === 'a' || tag === 'select' || tag === 'textarea') return
    if (e.target.closest('button') || e.target.closest('input')) return
    invoke('start_dragging')
  }, [])

  useEffect(() => {
    invoke('get_card_switch_secs').then(v => setCardSwitchSeconds(v)).catch(() => {})
    invoke('get_login_status').then(status => {
      if (status) {
        setIsLoggedIn(true)
        invoke('get_usage_data').then(data => {
          setUsageData(normalize(data))
        }).catch(() => {})
      }
    }).catch(() => {})
    listen('login-successful', () => {
      setIsLoggedIn(true)
      invoke('get_usage_data').then(data => {
        setUsageData(normalize(data))
      })
    })
    listen('usage-data-updated', (e) => {
      setUsageData(normalize(e.payload))
      setIsRefreshing(false)
      setIsSuccess(true)
      setTimeout(() => setIsSuccess(false), 1500)
    })
    listen('logged-out', () => {
      setIsLoggedIn(false)
      setUsageData([
        { title: "每5小时额度", percentage: "0%", resetTime: "加载中...", isHighUsage: false },
        { title: "每周额度", percentage: "0%", resetTime: "加载中...", isHighUsage: false }
      ])
    })
    listen('card-switch-interval-changed', (e) => {
      setCardSwitchSeconds(e.payload)
    })
    listen('api-error', (e) => {
      setIsRefreshing(false)
      console.error('API Error:', e.payload)
    })
  }, [])

  const normalize = (data) => [
    { title: "每5小时额度", percentage: data.hourly.percentage, resetTime: data.hourly.resetTime, isHighUsage: parseInt(data.hourly.percentage) > 80 },
    { title: "每周额度", percentage: data.weekly.percentage, resetTime: data.weekly.resetTime, isHighUsage: parseInt(data.weekly.percentage) > 80 }
  ]

  const handleLogout = useCallback(() => {
    invoke('logout')
  }, [])
  const handleTogglePin = useCallback(() => {
    invoke('toggle_always_on_top').then(pinned => setIsPinned(pinned))
  }, [])
  const handleRefresh = useCallback(() => {
    setIsRefreshing(prev => {
      if (prev) return true
      setIsSuccess(false)
      invoke('refresh_usage_data')
      setTimeout(() => setIsRefreshing(p => p ? false : p), 15000)
      return true
    })
  }, [])

  const nextCard = useCallback(() => {
    if (isAnimating) return
    setIsAnimating(true)
    setTimeout(() => { setCurrentCard(prev => (prev + 1) % 2); setIsAnimating(false) }, 300)
  }, [isAnimating])
  const prevCard = useCallback(() => {
    if (isAnimating) return
    setIsAnimating(true)
    setTimeout(() => { setCurrentCard(prev => (prev - 1 + 2) % 2); setIsAnimating(false) }, 300)
  }, [isAnimating])

  const switchTimerRef = useRef(null)
  useEffect(() => {
    if (switchTimerRef.current) clearInterval(switchTimerRef.current)
    if (cardSwitchSeconds > 0) {
      switchTimerRef.current = setInterval(() => { if (!isAnimating) nextCard() }, cardSwitchSeconds * 1000)
    }
    return () => clearInterval(switchTimerRef.current)
  }, [isAnimating, nextCard, cardSwitchSeconds])

  return (
    <div className="no-select" style={{ background: 'transparent' }} onMouseDown={handleDragStart}>
      {isLoggedIn ? (
        <UsageCard {...usageData[currentCard]}
          otherPercentage={usageData[(currentCard + 1) % 2].percentage}
          otherTitle={usageData[(currentCard + 1) % 2].title}
          onNext={nextCard} onPrev={prevCard}
          isPinned={isPinned} onTogglePin={handleTogglePin}
          isRefreshing={isRefreshing} isSuccess={isSuccess} onRefresh={handleRefresh}
          onOpenSettings={() => setShowSettings(true)} onLogout={handleLogout} />
      ) : (
        <LoginScreen />
      )}
      <SettingsDialog open={showSettings} onClose={() => setShowSettings(false)} />
    </div>
  )
}

export default App
