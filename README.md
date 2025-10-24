# PeachLeaf

크로스 플랫폼 마크다운 스티커 노트 애플리케이션

## 주요 기능

- **크로스 플랫폼 지원**: Windows, Linux, macOS에서 동작
- **마크다운 지원**: 마크다운 파일을 편집하고 미리보기
- **3가지 편집 모드**:
  - 편집 모드: 마크다운 구문으로 편집
  - 미리보기 모드: 렌더링된 마크다운 보기
  - 리치텍스트 모드: WYSIWYG 편집
- **자동 저장**: 500ms 디바운스로 자동 저장
- **커스터마이징**: 배경색과 글자색 변경 가능
- **자유로운 배치**: 스티커 이동 및 크기 조절
- **단축키 지원**: 편집 가능한 키보드 단축키

## 설치

```bash
# 의존성 설치
npm install

# 개발 모드 실행
npm run dev

# 빌드
npm run build

# 패키징
npm run package

# 모든 플랫폼 빌드
npm run package:all
```

## 사용법

### 기본 단축키

- `Cmd/Ctrl + M`: 모드 전환 (편집 → 미리보기 → 리치텍스트)
- `Cmd/Ctrl + Shift + N`: 새 스티커 생성

### 스티커 조작

- 상단 툴바를 드래그하여 이동
- 창 테두리를 드래그하여 크기 조절
- 🎨 버튼으로 색상 변경
- ✕ 버튼으로 스티커 닫기

## 기술 스택

- **Electron**: 크로스 플랫폼 데스크톱 앱
- **Svelte**: 경량 UI 프레임워크
- **TypeScript**: 타입 안전성
- **CodeMirror 6**: 마크다운 편집기
- **marked**: 마크다운 렌더링
- **electron-store**: 설정 저장

## 프로젝트 구조

```
peach-leaf/
├── src/
│   ├── main/           # Electron 메인 프로세스
│   │   ├── main.ts     # 앱 진입점, 창 관리
│   │   └── preload.ts  # IPC 브릿지
│   └── renderer/       # Svelte UI
│       ├── components/ # UI 컴포넌트
│       ├── App.svelte  # 루트 컴포넌트
│       └── main.ts     # 렌더러 진입점
├── package.json
└── vite.config.ts
```

## 라이선스

MIT
