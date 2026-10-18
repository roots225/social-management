import axios from 'axios';
import {
  DashboardStats,
  Publication,
  PublicationStatus,
  ScrapedItem,
  SocialAccount,
  Source,
} from '../types';

const api = axios.create({
  baseURL: '/api/v1',
  headers: {
    'Content-Type': 'application/json',
  },
});

export const apiClient = {
  // Stats Dashboard
  getStats: async (): Promise<DashboardStats> => {
    const { data } = await api.get<DashboardStats>('/dashboard/stats');
    return data;
  },

  // Sources
  getSources: async (): Promise<Source[]> => {
    const { data } = await api.get<Source[]>('/sources');
    return data;
  },

  createSource: async (source: Partial<Source>): Promise<Source> => {
    const { data } = await api.post<Source>('/sources', source);
    return data;
  },

  updateSource: async (id: string, source: Partial<Source>): Promise<Source> => {
    const { data } = await api.put<Source>(`/sources/${id}`, source);
    return data;
  },

  deleteSource: async (id: string): Promise<void> => {
    await api.delete(`/sources/${id}`);
  },

  scrapeSourceNow: async (id: string): Promise<any> => {
    const { data } = await api.post(`/sources/${id}/scrape-now`);
    return data;
  },

  // Scraped Items (Flux de Curation)
  getScrapedItems: async (params?: {
    status?: string;
    category?: string;
    limit?: number;
    skip?: number;
  }): Promise<ScrapedItem[]> => {
    const { data } = await api.get<ScrapedItem[]>('/scraped-items', { params });
    return data;
  },

  updateItemStatus: async (id: string, status: string): Promise<void> => {
    await api.put(`/scraped-items/${id}/status`, { status });
  },

  queueItem: async (
    id: string,
    payload: { priority?: number; custom_text?: string; target_account_ids?: string[] }
  ): Promise<Publication> => {
    const { data } = await api.post<Publication>(`/scraped-items/${id}/queue`, payload);
    return data;
  },

  // Publications (File d'attente)
  getPublications: async (params?: {
    status?: PublicationStatus;
    limit?: number;
    skip?: number;
  }): Promise<Publication[]> => {
    const { data } = await api.get<Publication[]>('/publications', { params });
    return data;
  },

  updatePublication: async (
    id: string,
    payload: {
      priority?: number;
      default_text?: string;
      media_urls?: string[];
      deliveries?: { account_id: string; custom_text?: string }[];
    }
  ): Promise<Publication> => {
    const { data } = await api.put<Publication>(`/publications/${id}`, payload);
    return data;
  },

  deletePublication: async (id: string): Promise<void> => {
    await api.delete(`/publications/${id}`);
  },

  retryPublication: async (id: string): Promise<any> => {
    const { data } = await api.post(`/publications/${id}/retry`);
    return data;
  },

  publishNow: async (id: string): Promise<any> => {
    const { data } = await api.post(`/publications/${id}/publish-now`);
    return data;
  },

  // Comptes Sociaux
  getSocialAccounts: async (): Promise<SocialAccount[]> => {
    const { data } = await api.get<SocialAccount[]>('/social-accounts');
    return data;
  },

  createSocialAccount: async (payload: any): Promise<SocialAccount> => {
    const { data } = await api.post<SocialAccount>('/social-accounts', payload);
    return data;
  },

  deleteSocialAccount: async (id: string): Promise<void> => {
    await api.delete(`/social-accounts/${id}`);
  },

  verifySocialAccount: async (id: string): Promise<any> => {
    const { data } = await api.post(`/social-accounts/${id}/verify`);
    return data;
  },
};
