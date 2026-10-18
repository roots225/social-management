export type SourceCategory = 'NEWS' | 'JOB_OFFER';

export interface ScraperConfig {
  item_container: string;
  title: string;
  url: string;
  summary?: string;
  image?: string;
  specific_fields?: Record<string, string>;
  render_js?: boolean;
  wait_for_selector?: string;
  wait_duration_ms?: number;
}

export interface Source {
  _id?: string;
  name: string;
  base_url: string;
  category: SourceCategory;
  scraper_config: ScraperConfig;
  cron_schedule: string;
  is_active: boolean;
  last_scraped_at?: string;
}

export type CurationStatus = 'NEW' | 'QUEUED' | 'REJECTED';

export type ItemMetadata =
  | {
      item_type: 'News';
      author?: string;
      category?: string;
    }
  | {
      item_type: 'JobOffer';
      company?: string;
      location?: string;
      contract_type?: string;
      expires_at?: string;
    };

export interface ScrapedItem {
  _id?: string;
  source_id: string;
  url_hash: string;
  original_url: string;
  title: string;
  summary?: string;
  image_url?: string;
  curation_status: CurationStatus;
  scraped_at: string;
  metadata?: ItemMetadata;
}

export type Platform = 'TWITTER' | 'FACEBOOK' | 'LINKEDIN' | 'TELEGRAM';

export interface AccountHealthStatus {
  is_valid: boolean;
  last_verified_at?: string;
  last_error?: string;
  token_expires_at?: string;
}

export interface SocialAccount {
  id: string;
  label: string;
  platform: Platform;
  is_active: boolean;
  auto_publish_categories: SourceCategory[];
  credentials: {
    auth_type: string;
    [key: string]: string;
  };
  health_status: AccountHealthStatus;
}

export type PublicationStatus =
  | 'PENDING'
  | 'PROCESSING'
  | 'PUBLISHED'
  | 'PARTIAL_FAILURE'
  | 'FAILED';

export type DeliveryStatus = 'PENDING' | 'SENT' | 'FAILED';

export interface Delivery {
  account_id: string;
  platform: Platform;
  custom_text?: string;
  status: DeliveryStatus;
  external_post_id?: string;
  published_at?: string;
  error_message?: string;
}

export interface Publication {
  _id?: string;
  scraped_item_id: string;
  status: PublicationStatus;
  priority: number;
  default_text: string;
  media_urls: string[];
  created_at: string;
  published_at?: string;
  deliveries: Delivery[];
}

export interface QuotaHistoryEntry {
  publication_id: string;
  slot: string;
  executed_at: string;
}

export interface DailyQuota {
  _id: string;
  published_count: number;
  max_limit: number;
  history: QuotaHistoryEntry[];
}

export interface DashboardStats {
  today: string;
  quota: {
    published_count: number;
    max_limit: number;
    remaining: number;
    is_full: boolean;
    history: QuotaHistoryEntry[];
  };
  next_slot: {
    label: string;
    target_time: string;
  };
  publications: {
    pending: number;
    published: number;
    failed: number;
  };
  social_accounts: {
    total: number;
    active: number;
    healthy: number;
    errors: number;
  };
  curation: {
    total: number;
    new: number;
    queued: number;
    rejected: number;
  };
  sources: {
    total: number;
    active: number;
  };
}
