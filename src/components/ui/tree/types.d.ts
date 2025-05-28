interface ITreeItem {
  id: string | null
  value: string
  name: string | null
  icon?: string | null
  children?: ITreeItem[] | null
  type: 'folder' | 'file'
  color?: string | null
  order?: number | null
  parentId?: string | null
}

interface IVaultGroupTreeItem {
  id?: string | null
  name: string
  icon?: string | null
  children?: IVaultGroupTreeItem[] | null
  desciption?: string | null
}
