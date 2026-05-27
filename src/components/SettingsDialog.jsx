import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { X } from 'lucide-react'

const SettingsDialog = ({ open, onClose }) => {
  const [refreshMinutes, setRefreshMinutes] = useState('3')
  const [cardSwitchDisabled, setCardSwitchDisabled] = useState(false)
  const [cardSwitchSeconds, setCardSwitchSeconds] = useState('30')

  useEffect(() => {
    if (open) {
      invoke('get_refresh_interval').then(v => setRefreshMinutes(String(Math.round(v / 60))))
      invoke('get_card_switch_secs').then(v => {
        if (v === 0) {
          setCardSwitchDisabled(true)
        } else {
          setCardSwitchDisabled(false)
          setCardSwitchSeconds(String(v))
        }
      })
    }
  }, [open])

  const handleSave = async () => {
    const mins = Math.max(0.5, parseFloat(refreshMinutes) || 3)
    const secs = cardSwitchDisabled ? 0 : Math.max(5, parseInt(cardSwitchSeconds) || 30)
    await invoke('set_refresh_interval', { seconds: Math.round(mins * 60) })
    await invoke('set_card_switch_secs', { seconds: secs })
    onClose()
  }

  if (!open) return null

  return (
    <div className="w-64 h-52 rounded-2xl overflow-hidden shadow-2xl flex flex-col absolute inset-0 z-50 no-select"
         style={{ backgroundColor: 'rgba(30, 30, 30, 0.95)' }}>
      <div className="drag-region flex items-center justify-between pt-3 pb-2 px-4">
        <span className="text-white text-xs font-bold">设置</span>
        <button onClick={onClose} className="no-drag w-5 h-5 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors">
          <X size={12} className="text-gray-400" />
        </button>
      </div>

      <div className="flex-1 flex flex-col justify-center px-4 space-y-3">
        <div>
          <label className="block text-gray-400 text-[10px] font-bold mb-1">数据刷新间隔（分钟）</label>
          <input
            type="number"
            min="0.5"
            step="0.5"
            value={refreshMinutes}
            onChange={e => setRefreshMinutes(e.target.value)}
            className="w-full h-6 rounded-md px-2 text-white text-[11px] outline-none border border-white/10 focus:border-blue-500 transition-colors"
            style={{ backgroundColor: 'rgba(255,255,255,0.08)' }}
          />
        </div>

        <div>
          <label className="block text-gray-400 text-[10px] font-bold mb-1">卡片切换间隔（秒）</label>
          <input
            type="number"
            min="5"
            step="1"
            disabled={cardSwitchDisabled}
            value={cardSwitchSeconds}
            onChange={e => setCardSwitchSeconds(e.target.value)}
            className="w-full h-6 rounded-md px-2 text-[11px] outline-none border border-white/10 focus:border-blue-500 transition-colors disabled:opacity-30"
            style={{ backgroundColor: 'rgba(255,255,255,0.08)', color: cardSwitchDisabled ? 'rgba(255,255,255,0.3)' : 'white' }}
          />
          <label className="flex items-center gap-1.5 mt-1 cursor-pointer">
            <input
              type="checkbox"
              checked={cardSwitchDisabled}
              onChange={e => setCardSwitchDisabled(e.target.checked)}
              className="w-3 h-3 accent-blue-500"
            />
            <span className="text-gray-400 text-[9px]">不自动切换卡片</span>
          </label>
        </div>
      </div>

      <div className="px-4 pb-3">
        <button onClick={handleSave}
                className="w-full h-6 rounded-lg text-white text-[11px] font-bold transition-colors hover:bg-blue-600"
                style={{ backgroundColor: '#3b82f6' }}>
          保存
        </button>
      </div>
    </div>
  )
}

export default SettingsDialog
